#!/usr/bin/env python3
"""Prove product-external-info-v4 differs from the frozen evaluator only by Groq provider wiring."""

from pathlib import Path
import subprocess
import sys

BASELINE = "e324ccbff6e818d205a734f06ccc8cac4b587588"
TARGET = "crates/reasoning-harness-cli/src/bin/product_external_info_v4.rs"
ALLOWED_ADDITIONS = {
    "use reasoning_harness_providers::GroqAdapter;",
    "    Groq,",
    '            Self::Groq => "groq",',
    "    Groq(GroqAdapter),",
    "            Provider::Groq => GroqAdapter::from_env(model).map(Self::Groq),",
    "            Self::Groq(adapter) => adapter,",
}


def baseline_text() -> str:
    proc = subprocess.run(
        ["git", "show", f"{BASELINE}:{TARGET}"],
        check=True,
        capture_output=True,
        text=True,
    )
    return proc.stdout


def normalize_current(text: str) -> str:
    lines = text.splitlines(keepends=True)
    removed = set()
    kept = []
    for line in lines:
        stripped = line.rstrip("\n")
        if stripped in ALLOWED_ADDITIONS:
            removed.add(stripped)
            continue
        kept.append(line)
    missing = ALLOWED_ADDITIONS - removed
    if missing:
        raise SystemExit(f"missing expected Groq provider-wiring additions: {sorted(missing)}")
    return "".join(kept)


current = Path(TARGET).read_text()
normalized = normalize_current(current)
baseline = baseline_text()
if normalized != baseline:
    import difflib

    diff = "".join(
        difflib.unified_diff(
            baseline.splitlines(keepends=True),
            normalized.splitlines(keepends=True),
            fromfile=f"{BASELINE}:{TARGET}",
            tofile=f"normalized:{TARGET}",
        )
    )
    sys.stderr.write(diff)
    raise SystemExit("frozen v4 evaluator changed outside the explicit Groq provider-wiring allowlist")
print("product-external-info-v4 frozen evaluator semantics unchanged; Groq provider wiring only")
