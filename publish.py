#!/usr/bin/env -S uv run
# /// script
# dependencies = [
#     "sh",
#     "tomlkit",
#     "structlog",
# ]
# ///

from dataclasses import dataclass
from datetime import datetime
from pathlib import Path
import sh
import tomlkit
import structlog
import argparse

structlog.configure(
    processors=[
        structlog.processors.add_log_level,
        structlog.processors.TimeStamper(fmt="%Y-%m-%d %H:%M:%S"),
        structlog.dev.ConsoleRenderer()
    ]
)
logger = structlog.get_logger(__name__)

@dataclass(frozen=True)
class GitRepo:
    branch: str = "dev"
    remote: str = "origin"

@dataclass(frozen=True)
class Project:
    name: str
    version: str
    path: Path
    repo: GitRepo = GitRepo()

    @property
    def manifest_path(self) -> Path:
        return self.path / "Cargo.toml"

    def update_manifest(self) -> None:
        manifest = tomlkit.parse(self.manifest_path.read_text(encoding="utf-8"))
        
        manifest["package"]["version"] = self.version
        manifest["package"]["name"] = f"abyss-{self.name}"
        
        self.manifest_path.write_text(tomlkit.dumps(manifest), encoding="utf-8")


if __name__ == "__main__":
    parser = argparse.ArgumentParser()
    parser.add_argument("--release", action="store_true")
    args = parser.parse_args()
    
    project = Project(
        name="snaps",
        version=datetime.now().strftime("%y.%-m.%-d"),
        path=Path(__file__).parent,
    )

    tag_name = f"v{project.version}"
    
    logger.info(f"Project: {project.name} {tag_name}.")
    project.update_manifest()
    logger.info("Cargo.toml updated.")

    with sh.pushd(project.path):
        sh.cargo("check")
        
        sh.git("add", ".")
        sh.git("commit", "-m", f"deploy-{datetime.now()}", "--allow-empty")
        sh.git("push", project.repo.remote, project.repo.branch)
        logger.info(f"Code pushed to {project.repo.branch} branch.")

        if args.release:
            logger.info("Managing tags and releases.")
    
            sh.cargo("build", "--release")
            sh.git("tag", "-f", tag_name)
            sh.git("push", project.repo.remote, tag_name, "--force")

            binary_name = f"abyss-{project.name}"
            binary_path = project.path / "target" / "release" / binary_name

            sh.gh(
                "release", "create", 
                tag_name, 
                str(binary_path), 
                "--title", f"{project.name} {tag_name}", 
                "--notes", f"{project.name} {tag_name}"
            )

            logger.info("Git release created/updated!")

