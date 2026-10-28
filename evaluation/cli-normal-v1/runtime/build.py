#!/usr/bin/env python3
"""Build the same fixed ordinary CLI host against both Engine arms."""
import importlib.util, sys
from pathlib import Path
HERE=Path(__file__).resolve().parent
HOST='dbad2dc25d3487819206582f1fad778d9af75da1'
sys.path.insert(1,str(HERE.parent))
spec=importlib.util.spec_from_file_location('preserved_paired_build',HERE.parent/'build.py')
paired=importlib.util.module_from_spec(spec);spec.loader.exec_module(paired)
# The underlying builder still hashes its own file, all source, exact patches,
# providers and binaries. The wrapper and HOST are separately in the sealed
# runtime inventory; checked_binaries also fixes this exact new host.
paired.HOST=HOST
if __name__=='__main__':paired.main()
