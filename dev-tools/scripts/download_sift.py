"""DEPRECATED: use `python scripts/download_ground_truth.py --datasets sift-1m`
(single entry point, idempotent, with manifest + --check).

This script pulled `ftp://ftp.irisa.fr/local/texmex/corpus/sift.tar.gz`, a host
that is dead, inside a `try/except` that only printed -- so it exited 0 whether
SIFT had been downloaded or not, and "downloaded" was indistinguishable from
"not downloaded" by exit code (FIND-664, FIND-824).

Kept as a redirect instead of deleted: `datasets/README.md` and
`scripts/verify_datasets.ps1` still name it as the action to take for SIFT-1M,
and a script that fails loudly with the replacement's command sends the reader
somewhere useful. It never touches the network and never exits 0.
"""

import sys

REPLACEMENT = "python scripts/download_ground_truth.py --datasets sift-1m"


def main() -> int:
    print("download_sift.py is DEPRECATED: its FTP host is dead.", file=sys.stderr)
    print(f"SIFT-1M is provisioned by: {REPLACEMENT}", file=sys.stderr)
    return 1


if __name__ == "__main__":
    sys.exit(main())
