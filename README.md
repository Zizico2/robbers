# Robbers

Run the Bevy development build with `cargo run`. The first build compiles the
engine; subsequent builds reuse it as a dynamic library.

The project pins a nightly Rust toolchain with Cranelift, uses LLVM for
dependencies, shares generic code, and links with clang and mold on x86-64
Linux. Rustup installs the pinned toolchain and component automatically. On
Arch, install `clang` and `mold` along with Bevy's Linux dependencies.

For a standalone release build, disable development dynamic linking:

```sh
cargo build --release --no-default-features
```

Changing the nightly toolchain can require a full rebuild and may expose
nightly compiler or Cranelift regressions. The configuration is intended for
native Linux development; cross-platform targets may need their own settings.
