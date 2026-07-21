import sh
import tomlkit
from pathlib import Path
from datetime import datetime

BRANCH = "main"

if __name__ == "__main__":
    now = datetime.now()
    version = now.strftime("%y.%-m.%-d")
    tag_name = f"v{version}"
    
    project_path = Path(__file__).parent.parent
    cargo_toml = project_path / "Cargo.toml"

    print(f"Project version: {version}")
    print(f"Project path: {project_path}")

    doc = tomlkit.parse(cargo_toml.read_text(encoding="utf-8"))
    doc["package"]["version"] = version
    cargo_toml.write_text(tomlkit.dumps(doc), encoding="utf-8")
    
    with sh.pushd(project_path):
        sh.cargo("build", "--release")
    
        sh.git("add", ".")
        sh.git("commit", "-m", f"ci-{now}", "--allow-empty")
        sh.git("push", "origin", BRANCH)

        try:
            sh.git("tag", "-d", tag_name)
        except:
            pass
        
        try:
            sh.git("push", "origin", "--delete", tag_name)
        except sh.ErrorReturnCode:
            pass

        sh.git("tag", tag_name)
        sh.git("push", "origin", tag_name)

        binary_path = project_path / "target" / "release" / doc["package"]["name"]

        sh.gh(
            "release", "create", 
            tag_name, 
            str(binary_path), 
            "--title", f"Abyss Snaps {tag_name}", 
            "--notes", f"Abyss Snaps {tag_name}",
            "--clobber"
        ) 

