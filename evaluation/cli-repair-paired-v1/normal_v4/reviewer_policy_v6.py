"""Source-enrolled immutable V6 safety rubric; no provider or credential calls.

Returns the exact independently committed safety rubric only after verifying
the two frozen arms' source artifacts against their native bundle records.
This is a local review-side prerequisite, not an alternative model verdict.
"""
from pathlib import Path
import hashlib
import json

POLICY_PATH = "evaluation/cli-repair-paired-v1/normal_v4/reviewer_safety_policy_v6.txt"
BUNDLE_CONTRACT = "reason-cli-normal-v3-native-bundle-v2"


def load_enrolled_v6_rubric(bootstrap):
    root = Path(bootstrap)
    raw_manifest = (root / "native-bundle.json").read_bytes()
    bundle = json.loads(raw_manifest)
    if (bundle.get("contract") != BUNDLE_CONTRACT
            or json.dumps(bundle,sort_keys=True,separators=(",",":"),
                          ensure_ascii=False,allow_nan=False).encode()!=raw_manifest):
        raise ValueError("v6_review_native_bundle_noncanonical")
    files = bundle.get("files")
    if type(files) is not dict:
        raise ValueError("v6_review_bundle_inventory_invalid")
    contents = []
    for arm in ("baseline", "candidate"):
        name = "source/" + arm + "/" + POLICY_PATH
        path = root / "bundle-files" / name
        if not path.is_file() or path.is_symlink():
            raise ValueError("v6_review_policy_missing_or_symlink")
        payload = path.read_bytes()
        record = files.get(name)
        if (type(record) is not dict or
                record.get("sha256") != hashlib.sha256(payload).hexdigest()):
            raise ValueError("v6_review_policy_not_native_bundle_enrolled")
        contents.append(payload)
    if contents[0] != contents[1]:
        raise ValueError("v6_review_arms_policy_disagree")
    try:
        policy=contents[0].decode("utf-8",errors="strict")
    except UnicodeError as exc:
        raise ValueError("v6_review_policy_invalid_utf8") from exc
    if ("NEGATIVE-CONTROL DISTINCTION:" not in policy
            or "Return UNKNOWN only" not in policy
            or len(policy)<1000):
        raise ValueError("v6_review_policy_content_invalid")
    return policy
