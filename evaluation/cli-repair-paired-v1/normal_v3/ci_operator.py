"""Local encrypted-CI transport for the appointed reviewer; never signs reviews."""
import argparse
import base64
import json
import lzma
import os
from pathlib import Path
import subprocess

from nacl.exceptions import CryptoError

from ci_exchange import MAX_ARCHIVE, MAX_PLAIN, Mailbox, messages, unseal
from contracts import canonical, digest, private_new_directory, require
from native_lane import private_bytes, private_snapshot


def mailbox(run_id, since, *, sender_id=None):
    token = os.environ.get("GH_TOKEN") or subprocess.check_output(["gh", "auth", "token"], timeout=15).decode().strip()
    return Mailbox(run_id=run_id, token=token, since=since, sender_id=sender_id)


def decode_packet(comments, run_id, stage, key, session, session_raw):
    valid = []
    for candidate in messages(comments, run_id, stage, sender_id=session["github_bot_id"]):
        try:
            raw = unseal(candidate, key, limit=MAX_ARCHIVE if stage == "final" else MAX_PLAIN)
            value = json.loads(raw)
            require(canonical(value) == raw and value["ci_session_sha256"] == digest(session_raw), "ci_packet_session_mismatch")
            if stage.startswith("job-"):
                expected = value["expected"]
                require(expected["evaluation_id"] == "ci-v3-" + str(run_id)
                        and value["packet"]["job_id"] == expected["job_id"], "ci_packet_job_mismatch")
            else:
                require(stage == "final" and value["evaluation_id"] == "ci-v3-" + str(run_id), "ci_final_session_mismatch")
            valid.append(raw)
        except (ValueError, TypeError, KeyError, CryptoError, lzma.LZMAError):
            continue
    require(len(valid) == 1, "ci_one_authenticated_packet_required")
    return valid[0]


def unpack(raw, output):
    value = json.loads(raw)
    require(canonical(value) == raw and set(value) == {"contract", "files"}
            and value["contract"] == "reason-cli-normal-v3-ci-private-files-v1" and type(value["files"]) is dict,
            "ci_archive_contract")
    decoded = {}
    for name, encoded in value["files"].items():
        require(type(name) is str and name and not Path(name).is_absolute() and ".." not in Path(name).parts
                and "." not in Path(name).parts and str(Path(name)) == name, "ci_archive_path_escape")
        require(type(encoded) is str, "ci_archive_encoding")
        decoded[name] = base64.b64decode(encoded, validate=True)
    output = Path(output).resolve()
    private_new_directory(output)
    for name, data in decoded.items():
        path = output / name
        path.parent.mkdir(mode=0o700, parents=True, exist_ok=True)
        private_bytes(path, data)


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("action", choices=("archive", "receive", "send"))
    parser.add_argument("--key", type=Path)
    parser.add_argument("--input", type=Path)
    parser.add_argument("--output", type=Path)
    parser.add_argument("--run-id", type=int)
    parser.add_argument("--stage")
    parser.add_argument("--since")
    parser.add_argument("--session", type=Path)
    args = parser.parse_args()
    if args.action == "archive":
        require(args.key is not None and args.input is not None and args.output is not None, "ci_archive_arguments")
        require(args.input.stat().st_size <= MAX_ARCHIVE + 65536, "ci_archive_ciphertext_limit")
        raw = unseal(args.input.read_bytes(), private_snapshot(args.key), limit=MAX_ARCHIVE)
        unpack(raw, args.output)
    else:
        require(args.run_id is not None and args.stage is not None and args.since is not None, "ci_message_arguments")
        require(args.session is not None, "ci_session_required")
        session_raw = private_snapshot(args.session)
        session = json.loads(session_raw)
        require(session["run_id"] == args.run_id, "ci_message_session_mismatch")
        box = mailbox(args.run_id, args.since)
        if args.action == "send":
            require(args.input is not None and args.session is not None, "ci_send_arguments")
            box.publish(args.stage, private_snapshot(args.input), bytes.fromhex(session["inbox_public_key_hex"]))
        else:
            require(args.key is not None and args.output is not None, "ci_receive_arguments")
            raw = decode_packet(box.comments(), args.run_id, args.stage, private_snapshot(args.key), session, session_raw)
            private_bytes(args.output, raw)
            print(json.dumps({"stage": args.stage, "packet_sha256": digest(raw)}))


if __name__ == "__main__":
    main()
