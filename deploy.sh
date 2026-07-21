#!/usr/bin/env bash
cd "$(dirname "$0")"
uv run --project ci ci/deploy.py
