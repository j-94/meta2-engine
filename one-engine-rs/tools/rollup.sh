#!/bin/bash
set -euo pipefail
python3 tools/bundles.py
python3 tools/check_policy.py || true
