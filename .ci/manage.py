#!/usr/bin/env -S uv run --script
# /// script
# requires-python = "==3.12.*"
# dependencies = [
#   "click==8.1.8",
#   "sh==2.2.2",
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


def run(cmd, *args, **kwargs):
    """Run a sh command; exit on non-zero status.
    Uses _fg when no other special arguments are present.
    """
    try:
        if kwargs:
            cmd(*args, **kwargs)
        else:
            cmd(*args, _fg=True)
    except sh.ErrorReturnCode as e:
        sys.exit(e.exit_code)


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
        run(sh.docker.buildx.build, *args)

    if README_SCRIPT.exists():
        # _in cannot be combined with _fg
        run(
            sh.Command(str(README_SCRIPT)),
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
        run(sh.git.add, ".")
        run(sh.git.commit, "-m", commit_msg, "--allow-empty")

        if push:
            run(sh.git.push, remote, branch, "--force")

        run(sh.git.tag, "-a", tag, "-m", f"Release {tag}", "--force")
        if push:
            run(sh.git.push, remote, tag, "--force")

    if not BINARY_PATH.exists():
        sys.exit(1)

    title = f"Release {tag}"
    notes = title

    if sh.which("gh"):
        try:
            run(sh.gh.release.create, tag, str(BINARY_PATH),
                "--title", title, "--notes", notes)
        except SystemExit:
            run(sh.gh.release.upload, tag, str(BINARY_PATH), "--clobber")
            run(sh.gh.release.edit, tag, "--title", title)
    elif sh.which("glab"):
        run(sh.glab.release.create, tag, str(BINARY_PATH),
            "--name", title, "--notes", notes, "--overwrite")


if __name__ == "__main__":
    cli()
