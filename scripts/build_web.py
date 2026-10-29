#!/usr/bin/env python3
"""Build the shared Bevy app and bundle a self-contained static web directory."""
import argparse
import os
from pathlib import Path
import shutil
import subprocess
import tomllib

ROOT = Path(__file__).resolve().parents[1]


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--release", action="store_true")
    args = parser.parse_args()
    lock = tomllib.loads((ROOT / "Cargo.lock").read_text())
    version = next(p["version"] for p in lock["package"] if p["name"] == "wasm-bindgen")
    bindgen = os.environ.get("WASM_BINDGEN", "wasm-bindgen")
    try:
        installed = subprocess.check_output([bindgen, "--version"], text=True).strip()
    except FileNotFoundError:
        installed = "missing"
    if installed != f"wasm-bindgen {version}":
        raise SystemExit(f"Need wasm-bindgen {version} (found {installed}).\n"
                         f"Install: rtk cargo install wasm-bindgen-cli --version {version} --locked")
    env = dict(os.environ, CARGO_PROFILE_DEV_CODEGEN_BACKEND="llvm", CARGO_PROFILE_DEV_DEBUG="0")
    command = ["cargo", "build", "--locked", "--target", "wasm32-unknown-unknown",
               "--no-default-features", "--features", "graphics", "--bin", "robbers"]
    if args.release:
        command.append("--release")
    subprocess.run(command, cwd=ROOT, env=env, check=True)
    target = Path(env.get("CARGO_TARGET_DIR", ROOT / "target"))
    if not target.is_absolute():
        target = ROOT / target
    output = target / "web"
    output.mkdir(parents=True, exist_ok=True)
    profile = "release" if args.release else "debug"
    wasm = target / f"wasm32-unknown-unknown/{profile}/robbers.wasm"
    subprocess.run([bindgen, "--target", "web", "--out-dir", str(output), str(wasm)], check=True)
    shutil.copy2(ROOT / "web/index.html", output / "index.html")
    shutil.copytree(ROOT / "assets", output / "assets", dirs_exist_ok=True)
    print(f"Web build: {output}\nServe: rtk proxy python3 -m http.server 8000 --directory target/web")


if __name__ == "__main__":
    main()
