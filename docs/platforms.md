# Android and web previews

These are tentative ports of the current prototype, using the same game,
command dispatcher, meshes, PNG assets and Bevy UI as the desktop application.
Android is an arm64 NativeActivity APK for Android 8+ (API 26), with sensor
landscape orientation. It can rotate between the two landscape orientations.
It is a sideloadable debug build, not a Play Store release.

Android disables wgpu shader debug metadata while retaining its other renderer
settings. Some PowerVR Vulkan drivers abort on that metadata during shader
compilation (`spvcompiler: unexpected source language`), even in a debug-signed
APK. This is a workaround for that startup crash; device verification is still
required. See the [matching driver report](https://github.com/CyberTimon/RapidRAW/issues/1723).

The compact layout is selected below 650 logical pixels in height. It keeps
controls at normal text size with 44-pixel buttons, and **Notes** opens a
scrollable overlay over the board. Drag the evidence to scroll and use **Back
to map** to close it. Ending a turn closes notes and hides the next player's
notebook until **Begin turn**. The desktop inspector remains available in taller
windows. The browser displays a rotation prompt in portrait.

## Build the APK

Install the Android SDK, build tools 35.0.0, platform android-35, and NDK
28.2.13676358 using Android Studio or the official command-line tools. Set
`ANDROID_HOME` to the SDK directory and `ANDROID_NDK_ROOT` to that NDK directory.
Java is also required for APK signing.

```sh
rtk proxy rustup target add aarch64-linux-android
rtk cargo install cargo-apk --version 0.10.0 --locked
rtk proxy python3 scripts/build_android.py
```

The output is `target/debug/apk/robbers.apk`. The script uses LLVM for cross
compilation, overriding the repository's Linux Cranelift development backend.
It bundles the assets and cargo-apk signs with the local Android debug key.
No device or emulator is needed to build it.

To install on an attached device with USB debugging enabled:

```sh
rtk proxy adb install -r target/debug/apk/robbers.apk
```

## Build and serve the browser version

Python 3.11+ and the wasm-bindgen CLI are required. The bindgen CLI version must
match `Cargo.lock`; the build script checks this before compiling.

```sh
rtk proxy rustup target add wasm32-unknown-unknown
rtk cargo install wasm-bindgen-cli --version 0.2.129 --locked
rtk proxy python3 scripts/build_web.py
rtk proxy python3 -m http.server 8000 --directory target/web
```

Open <http://localhost:8000> in a WebGL 2 browser. Use **Play**, or **Play
fullscreen** to request fullscreen and landscape orientation when supported.
Use `--release` when building an export for distribution; development WASM is
substantially larger. The entire `target/web` directory must be served together,
including `assets/`, the JavaScript glue and the `.wasm` file.

The **Mobile preview** GitHub Actions workflow builds and uploads both the
debug-signed APK and a release web bundle on pull requests or manual runs.

## Verification and remaining device checks

Browser validation should cover landscape sizes 640×360, 844×390 and 915×412,
a tablet/desktop size, and portrait-to-landscape rotation. Enable mobile/touch
emulation and verify:

- Tap an adjacent hex: the investigator moves and spends exactly one action.
- Tap another investigator: the active player stays unchanged.
- Investigate, open Notes, and drag a notebook containing several clues.
- End the turn while Notes is open: the overlay closes and notes stay hidden.
- Begin the next turn and verify that only that investigator's notes appear.
- Sketch, undo and clear with touch; the pawn and action budget stay unchanged.

For scripted Chromium checks at DPR 2, also launch Chromium with
`--force-device-scale-factor=2`. Otherwise Chromium's emulated DPR can disagree
with its device-pixel ResizeObserver measurements, which Winit uses to size the
canvas. Touch tests should include a realistic press duration across rendered
frames, rather than a zero-duration synthetic tap on a newly created pointer.

Browser testing covers shared UI, sizing and touch handling. It cannot establish
Android renderer compatibility, device cutout/navigation-bar behaviour, GPU
performance, or activity suspend/resume behaviour. Those need a physical-device
pass before treating the Android port as ready.

Platform references: [Bevy mobile example](https://github.com/bevyengine/bevy/tree/v0.20.0/examples/mobile),
[cargo-apk configuration](https://github.com/rust-mobile/cargo-apk),
and [Bevy Window canvas options](https://docs.rs/bevy/0.20.0/bevy/window/struct.Window.html).
