#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="${CARGO_MANIFEST_DIR:-$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)}"
ANDROID_DIR="$ROOT_DIR/android"

if [[ ! -d "$ANDROID_DIR" ]]; then
  echo "Android project directory not found: $ANDROID_DIR" >&2
  exit 1
fi

ANDROID_HOME_VALUE="${ANDROID_HOME:-${ANDROID_SDK_ROOT:-}}"
if [[ -z "$ANDROID_HOME_VALUE" ]]; then
  echo "ANDROID_HOME or ANDROID_SDK_ROOT is not set. Please export it before running this command." >&2
  exit 1
fi

if [[ -z "${ANDROID_NDK_HOME:-${NDK_HOME:-}}" ]]; then
  NDK_DIR="$ANDROID_HOME_VALUE/ndk"
  if [[ -d "$NDK_DIR" ]]; then
    ANDROID_NDK_HOME="$(find "$NDK_DIR" -mindepth 1 -maxdepth 1 -type d | sort -V | tail -1)"
    export ANDROID_NDK_HOME
  fi
fi

if [[ -z "${ANDROID_NDK_HOME:-${NDK_HOME:-}}" ]]; then
  echo "Android NDK not found. Install it via Android Studio SDK Manager or set ANDROID_NDK_HOME." >&2
  exit 1
fi

if ! command -v cargo-ndk >/dev/null 2>&1; then
  echo "cargo-ndk is required. Install it with: cargo install cargo-ndk" >&2
  exit 1
fi

ADB_BIN="${ADB:-}"
if [[ -z "$ADB_BIN" ]]; then
  if [[ -x "$ANDROID_HOME_VALUE/platform-tools/adb" ]]; then
    ADB_BIN="$ANDROID_HOME_VALUE/platform-tools/adb"
  elif command -v adb >/dev/null 2>&1; then
    ADB_BIN="$(command -v adb)"
  else
    echo "adb was not found. Install Android platform-tools or set ADB/ANDROID_HOME." >&2
    exit 1
  fi
fi

APK_PATH="$ANDROID_DIR/app/build/outputs/apk/debug/app-debug.apk"

echo "Building Android native library..."
cargo ndk -t arm64-v8a -P 26 -o "$ANDROID_DIR/app/src/main/jniLibs" build --lib

echo "Building Android debug APK..."
(
  cd "$ANDROID_DIR"
  bash gradlew app:assembleDebug
)

if [[ ! -f "$APK_PATH" ]]; then
  echo "APK was not produced: $APK_PATH" >&2
  exit 1
fi

echo "Installing APK on emulator..."
"$ADB_BIN" install -r "$APK_PATH"

echo "Launching app on the emulator..."
if ! "$ADB_BIN" shell am start -n com.meguineapig.app/android.app.NativeActivity 2>/dev/null; then
  "$ADB_BIN" shell am start -a android.intent.action.MAIN -c android.intent.category.LAUNCHER com.meguineapig.app 2>/dev/null || true
fi

echo "Done. The app should appear on the emulator shortly."
