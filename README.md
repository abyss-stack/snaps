# Abyss Snaps

**Abyss-snaps** is a filesystem state machine and orchestrator for Btrfs.

## Features
* **Recipe Snapshots**: Snapshots and rolls back only paths marked as `tracked` in the provided recipe.
* **Untracked Paths**: Paths outside the `tracked` list are never snapshot and stay live.
* **Single Volume Only**: Supports only one Btrfs disk volume (`subvolid=5`). No standalone subvolumes.
* **Orchestration**: Includes compact flags for easy external control.
* **JSON Output**: Output uses a strict JSON contract and is easy to parse.
