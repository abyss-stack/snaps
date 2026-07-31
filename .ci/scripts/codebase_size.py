#!/usr/bin/env -S uv run --script
# /// script
# requires-python = "==3.12.*"
# dependencies = [
#     "dirstree==0.0.12",
#     "structlog==26.1.0",
#     "pygments==2.20.0",
# ]
# ///

# For strict typesafety inside class constructors.
from __future__ import annotations

PROJECT_NAME = "snaps"

from dirstree import Crawler
from pathlib import Path
from dataclasses import dataclass

from pygments.lexers import get_lexer_by_name
from pygments.token import Token
import pygments

import structlog
import sys
import json

structlog.configure(
    processors=[
        structlog.processors.add_log_level,
        structlog.processors.TimeStamper(fmt="%Y-%m-%d %H:%M:%S"),
        structlog.dev.ConsoleRenderer()
    ],
    logger_factory=structlog.PrintLoggerFactory(file=sys.stderr)
)

logger = structlog.get_logger(__name__)

class AppError(Exception):
    pass

def get_project_root(start: Path, name: str) -> Path | None:
    current = start.resolve()
    while current != current.parent:
        if current.name == name:
            return current
        current=current.parent
    return None

@dataclass(frozen=True)
class ProjectLayout:
    script_path: Path
    root: Path

    @classmethod
    def parse(cls) -> ProjectLayout:
        script_path = Path(__file__)
        root = get_project_root(script_path, PROJECT_NAME)

        if root is not None:
            return ProjectLayout(
                script_path,
                root,
            )

        raise AppError("Project root is None.")

def count_codelines(file_path: Path) -> int | None:
    try:
        content = file_path.read_text(encoding="utf-8")
    except Exception as e:
        logger.error(f"Read source file error: {e}")
        return None

    lexer = get_lexer_by_name("rust")
    codelines = 0

    for line in content.splitlines():
        # Guard skip for empty lines.
        if not line.strip():
            continue

        is_codeline = False

        for token_type, token_value in pygments.lex(line, lexer):
            # Skip comments and whitespace symbols.
            if token_type in Token.Comment or token_type in Token.Text:
                continue

            if token_value.strip():
                is_codeline = True
                break

        if is_codeline:
            codelines += 1

    return codelines
             

def main():
    try:
        project = ProjectLayout.parse()
    except AppError as e:
        logger.error(e)
        return

    src_path = project.root / "src"
    metrics = {}
    
    for path in Crawler(src_path, extensions = [".rs"]):
        file_path = Path(path)

        if (codelines:=count_codelines(file_path)) is not None:
            relative_path = file_path.relative_to(src_path)
            file_bytes = file_path.stat().st_size
            metrics[str(relative_path)] = {
                "loc": codelines,
                "bytes": file_bytes
            }
            logger.info("Parsed file", file=str(relative_path), loc=codelines, bytes=file_bytes)
    

    sorted_metrics = dict(
        sorted(metrics.items(), key=lambda item: item[1]["loc"], reverse=True)
    )

    print(json.dumps(sorted_metrics), file=sys.stdout)


if __name__ == "__main__":
    main()

