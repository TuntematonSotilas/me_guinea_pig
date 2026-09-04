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

## Run Android
```bash
cargo android
```

Build an installable release APK without installing it:
```bash
cargo android-build
```

The release APK is signed with the Android debug key for local testing. It must
use a private release keystore before being published.

## Android build notes
Prerequisites:
- Rust target: `aarch64-linux-android`
- Android SDK + NDK
- Java runtime
- install `cargo-ndk` 