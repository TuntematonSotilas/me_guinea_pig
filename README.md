# Me, Guinea Pig

A 2D mobile game prototype built with Bevy.

## Current prototype
- tap-to-move gameplay
- simple day/night cycle
- basic colored shapes for player and room
- basic needs system

## Android build setup
The project now includes a minimal Android project structure under [android](android) and Cargo target configuration for Android.

## Run
```bash
cargo run
```

## Android build notes
Prerequisites:
- Rust target: `aarch64-linux-android`
- Android SDK + NDK
- Java runtime
- `cargo-ndk` (installed and ready)

Recommended build flow:
```bash
cargo ndk -t arm64-v8a -o android/app/src/main/jniLibs build
```

If the NDK toolchain is installed, the following check should work:
```bash
rustup target add aarch64-linux-android
cargo check --target aarch64-linux-android
```

The Android packaging path is now wired for `cargo-ndk` and can be used once the NDK toolchain is available.
