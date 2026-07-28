#!/usr/bin/env -S uv run
# /// script
# requires-python = ">=3.12.0, <3.13.0"
# dependencies = [
#     "sh==2.2.2",
# ]
# ///

from contextlib import chdir
from dataclasses import dataclass
from pathlib import Path
import sys
import sh

TARGET_VERSION = '2026.07.29-1'

@dataclass
class GitConfig:
    remote: str = "origin"
    branch: str = "dev"

def main():
    # Setup configuration Data Transfer Object (DTO)
    git_cfg = GitConfig(remote="origin", branch="dev")

    ci_dir = Path(__file__).parent.resolve()
    project_root = ci_dir.parent

    stats_script = ci_dir / "stats.py"
    output_img = ci_dir / "artifact" / "codebase_size.png"
    dockerfile = ci_dir / "Dockerfile"
    artifact_dir = ci_dir / "artifact"
    binary_path = artifact_dir / "abyss-snaps"

    if not stats_script.exists():
        print(f"Error: Script not found at {stats_script}", file=sys.stderr)
        sys.exit(1)

    run_stats = sh.Command(str(stats_script))
    try:
        run_stats("--path", project_root, "--output", output_img, _fg=True)
        print(f"Chart saved to: {output_img}")
    except sh.ErrorReturnCode as e:
        print(f"Stats error: {e}", file=sys.stderr)
        sys.exit(e.exit_code)

    if not dockerfile.exists():
        print(f"Error: Dockerfile not found at {dockerfile}", file=sys.stderr)
        sys.exit(1)

    # 1. Compile static binary via Docker and inject current version
    artifact_dir.mkdir(exist_ok=True)
    try:
        with chdir(project_root):
            sh.docker.buildx.build(
                "-f", dockerfile, 
                "--target", "exporter", 
                "--build-arg", f"VERSION={TARGET_VERSION}",
                "-o", artifact_dir, 
                ".", _fg=True
            )
        print(f"Binary in: {artifact_dir}")
    except sh.ErrorReturnCode as e:
        print(f"Docker build error: {e}", file=sys.stderr)
        sys.exit(e.exit_code)

    if not binary_path.exists():
        print(f"Error: Compiled binary not found at {binary_path}", file=sys.stderr)
        sys.exit(1)

    # 2. Handle Git operations using parameters from the config DTO
    try:
        with chdir(project_root):
            print(f"Adding modifications and pushing to {git_cfg.remote}/{git_cfg.branch}...")
            
            sh.git.add(".", _fg=True)
            
            # Catching error if working tree is clean to prevent pipeline failure
            try:
                sh.git.commit("-m", f"chore(release): version {TARGET_VERSION} update", _fg=True)
            except sh.ErrorReturnCode:
                print("No changes to commit (working tree clean).")

            sh.git.push(git_cfg.remote, git_cfg.branch, _fg=True)
            
            # Create an annotated tag and push it to the configured remote
            print(f"Tagging release: v{TARGET_VERSION}")
            sh.git.tag("-a", f"v{TARGET_VERSION}", "-m", f"Release v{TARGET_VERSION}", _fg=True)
            sh.git.push(git_cfg.remote, f"v{TARGET_VERSION}", _fg=True)

    except sh.ErrorReturnCode as e:
        print(f"Git execution error: {e}", file=sys.stderr)
        sys.exit(e.exit_code)

    # 3. Publish release with compiled architecture asset
    print(f"Creating remote platform release for v{TARGET_VERSION}...")
    try:
        # Deploy using GitHub CLI if available
        if sh.which("gh"):
            sh.gh.release.create(
                f"v{TARGET_VERSION}",
                str(binary_path),
                "--title", f"Release v{TARGET_VERSION}",
                "--notes", f"Release v{TARGET_VERSION}",
                _fg=True
            )
            print("GitHub Release successfully created.")
        
        # Fallback to GitLab CLI if available
        elif sh.which("glab"):
            sh.glab.release.create(
                f"v{TARGET_VERSION}",
                str(binary_path),
                "--name", f"Release v{TARGET_VERSION}",
                "--notes", f"Release v{TARGET_VERSION}",
                _fg=True
            )
            print("GitLab Release successfully created.")
            
        else:
            print("Warning: Neither 'gh' nor 'glab' CLI discovered in PATH. Binary asset upload bypassed.", file=sys.stderr)

    except sh.ErrorReturnCode as e:
        print(f"Platform release platform error: {e}", file=sys.stderr)
        sys.exit(e.exit_code)

if __name__ == "__main__":
    main()
