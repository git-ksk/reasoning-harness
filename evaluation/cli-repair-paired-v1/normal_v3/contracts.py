"""Prospective v3 ordinary-CLI measurement contracts. No live/runtime admission."""
import hashlib
import json
import re
from pathlib import Path

PLAN_ID = "reason-cli-normal-v3-capture-manifest-v1"
OBS_ID = "reason-cli-normal-v3-observation-v1"
PACKET_ID = "reason-cli-normal-v3-review-packet-v1"
REVIEW_ID = "reason-cli-normal-v3-independent-review-v1"
SCORE_ID = "reason-cli-normal-v3-score-v1"
PHASES = frozenset((
    "initial_candidate", "candidate_regeneration", "investigation_plan",
    "investigation_intent", "investigation_action", "final_render",
))
FILES = ("cli.json", "trace.json", "http.json", "capture.json", "sources.json")
HEX = re.compile(r"^[a-f0-9]{64}$")


def require(condition, reason):
    if not condition:
        raise ValueError(reason)


def digest(blob):
    return hashlib.sha256(blob).hexdigest()


def canonical(value):
    return json.dumps(value, ensure_ascii=False, sort_keys=True, separators=(",", ":")).encode()


def keyed_digest(value):
    return digest(canonical(value))


def sha_field(value, label):
    require(isinstance(value, str) and HEX.fullmatch(value) is not None, label)
    return value


def read_json(path):
    return json.loads(Path(path).read_bytes())


def exclusive_json(path, value):
    path = Path(path)
    with path.open("x", encoding="utf-8") as stream:
        json.dump(value, stream, ensure_ascii=False, indent=2, sort_keys=True)
        stream.write("\n")


def strict_keys(value, allowed, required, name):
    require(type(value) is dict, name + "_not_object")
    require(set(required) <= value.keys(), name + "_missing_keys")
    require(value.keys() <= set(allowed), name + "_unexpected_keys")
    return value


def safe_id(value, name):
    require(
        type(value) is str
        and 0 < len(value) <= 120
        and re.fullmatch(r"[A-Za-z0-9][A-Za-z0-9._-]*", value) is not None
        and value not in (".", ".."),
        name,
    )
    return value


def unsigned(value, name):
    require(type(value) is int and value >= 0, name)
    return value


def positive(value, name):
    require(type(value) is int and value > 0, name)
    return value
