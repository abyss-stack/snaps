#!/usr/bin/env -S uv run --script
# /// script
# requires-python = "==3.12.*"
# dependencies = [
# "click==8.1.8",
# "sh==2.2.2",
# ]
# ///
import json
import sys
from contextlib import chdir
from datetime import datetime
from pathlib import Path

import click
import sh

CI_DIR = Path(__file__).parent.resolve()
PROJECT_ROOT = CI_DIR.parent
CONFIG_PATH = CI_DIR / "config.json"
DOCKERFILE = CI_DIR / "Dockerfile"
ARTIFACT_DIR = PROJECT_ROOT / "dist"
BINARY_PATH = ARTIFACT_DIR / "abyss-snaps"
README_SCRIPT = CI_DIR / "build_readme.py"


def load_config() -> dict:
    if not CONFIG_PATH.exists():
        sys.exit(1)
    return json.loads(CONFIG_PATH.read_text())


@click.group()
def cli():
    pass


@cli.command()
@click.option("--no-cache", is_flag=True)
def build(no_cache: bool):
    cfg = load_config()
    version = cfg["version"]
    ARTIFACT_DIR.mkdir(exist_ok=True)

    args = [
        "-f", str(DOCKERFILE),
        "--target", "exporter",
        "--build-arg", f"VERSION={version}",
        "-o", str(ARTIFACT_DIR),
        ".",
    ]
    if no_cache:
        args.insert(0, "--no-cache")

    with chdir(PROJECT_ROOT):
        sh.docker.buildx.build(*args, _fg=True)

    if README_SCRIPT.exists():
        sh.Command(str(README_SCRIPT))(
            _in=version,
            _out=sys.stdout,
            _err=sys.stderr,
        )


@cli.command()
@click.option("--push", is_flag=True)
def release(push: bool):
    cfg = load_config()
    version = cfg["version"]
    remote = cfg["remote"]
    branch = cfg["branch"]
    tag = f"v{version}"
    timestamp = datetime.now().strftime("%Y-%m-%dT%H:%M:%S")
    commit_msg = f"deploy-{timestamp}"

    with chdir(PROJECT_ROOT):
        sh.git.add(".", _fg=True)
        sh.git.commit("-m", commit_msg, "--allow-empty", _fg=True)
        if push:
            sh.git.push(remote, branch, "--force", _fg=True)
        sh.git.tag("-a", tag, "-m", f"Release {tag}", "--force", _fg=True)
        if push:
            sh.git.push(remote, tag, "--force", _fg=True)

    if not BINARY_PATH.exists():
        sys.exit(1)

    title = f"Release {tag}"
    notes = title

    if sh.which("gh"):
        try:
            sh.gh.release.create(
                tag, str(BINARY_PATH),
                "--title", title, "--notes", notes,
                _fg=True,
            )
        except sh.ErrorReturnCode:
            sh.gh.release.upload(tag, str(BINARY_PATH), "--clobber", _fg=True)
            sh.gh.release.edit(tag, "--title", title, _fg=True)
    elif sh.which("glab"):
        sh.glab.release.create(
            tag, str(BINARY_PATH),
            "--name", title, "--notes", notes, "--overwrite",
            _fg=True,
        )


if __name__ == "__main__":
    cli()
