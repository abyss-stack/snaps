pub const DEFAULT_CONFIG: &'static str = r#"
{
  "mounts": [
    {
      "device": "LABEL=ROOT_PART",
      "mountpoint": "/",
      "fstype": "btrfs",
      "options": ["strictatime"],
      "subvolume": "@",
      "dump": 0,
      "pass": 0,
      "is_state": true
    },
    {
      "device": "LABEL=ROOT_PART",
      "mountpoint": "/mnt/btrfs-root",
      "fstype": "btrfs",
      "options": ["strictatime", "subvolid=5"],
      "subvolume": null,
      "dump": 0,
      "pass": 0,
      "is_state": false
    },
    {
      "device": "LABEL=ROOT_PART",
      "mountpoint": "/.abyss-snaps",
      "fstype": "btrfs",
      "options": ["strictatime"],
      "subvolume": "@abyss-snaps",
      "dump": 0,
      "pass": 0,
      "is_state": false
    },
    {
      "device": "LABEL=ROOT_PART",
      "mountpoint": "/home",
      "fstype": "btrfs",
      "options": ["strictatime"],
      "subvolume": "@home",
      "dump": 0,
      "pass": 0,
      "is_state": true
    },
    {
      "device": "tmpfs",
      "mountpoint": "/tmp",
      "fstype": "tmpfs",
      "options": ["defaults", "nosuid", "nodev"],
      "subvolume": null,
      "dump": 0,
      "pass": 0,
      "is_state": false
    }
  ],
  "state": {
    "bootable_subvolume": "@",
    "snaps_root": "/mnt/btrfs-root/@abyss-snaps"
  }
}
"#;

pub const GREET: &'static str = r#"
  ____  ____   __ __  _____ _____        _____ ____    ____  ____    _____
 /    ||    \ |  |  |/ ___// ___/       / ___/|    \  /    ||    \  / ___/
|  o  ||  o  )|  |  (   \_(   \_  _____(   \_ |  _  ||  o  ||  o  )(   \_
|     ||     ||  ~  |\__  |\__  ||     |\__  ||  |  ||     ||   _/  \__  |
|  _  ||  O  ||___, |/  \ |/  \ ||_____|/  \ ||  |  ||  _  ||  |    /  \ |
|  |  ||     ||     |\    |\    |       \    ||  |  ||  |  ||  |    \    |
|__|__||_____||____/  \___| \___|        \___||__|__||__|__||__|     \___|

Part of abyss-stack tools: a state machine for Btrfs subvolumes (Static/Dynamic).
It implements a unique way to orchestrate your data, eliminating such common problems
as "My /home is from the future!" or "I've rolled back my database!".

You need to write your own rules in a config file, which is the single source of truth.
Abyss-snaps has a very simple UX, thanks to its config-driven architecture.
"#;
