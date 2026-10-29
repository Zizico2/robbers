#!/usr/bin/env python3
"""Build a debug-signed arm64 Android APK using cargo-apk and the installed SDK/NDK."""
import os
from pathlib import Path
import subprocess

ROOT = Path(__file__).resolve().parents[1]


def main():
    # The repository's Cranelift dev backend is for Linux; cross targets use LLVM.
    env = dict(os.environ, CARGO_PROFILE_DEV_CODEGEN_BACKEND="llvm", CARGO_PROFILE_DEV_DEBUG="0")
    subprocess.run(["cargo", "apk", "build", "--package", "robbers-android", "--lib"],
                   cwd=ROOT, env=env, check=True)
    target = Path(env.get("CARGO_TARGET_DIR", ROOT / "target"))
    print(f"APK: {target / 'debug/apk/robbers.apk'}")


if __name__ == "__main__":
    main()
