#!/usr/bin/env bash
cd "$(dirname "$0")"
uv run --project ci ci/gen_readme.py
uv run --project ci ci/deploy.py
echo "Done!"
