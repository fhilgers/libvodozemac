#!/usr/bin/env bash

# SPDX-FileCopyrightText: 2025 Felix Hilgers <contact@fhilgers.com>
#
# SPDX-License-Identifier: Apache-2.0

set -euo pipefail

PROFILE="release"
PROFILE_PATH="release"
WASM_PATH="$PWD/target/wasm32-unknown-unknown/$PROFILE_PATH/vodozemac.wasm"

rm -rf dist

cargo clean
cargo build --target x86_64-unknown-linux-gnu --profile=$PROFILE
cargo build --target aarch64-unknown-linux-gnu --profile=$PROFILE
cargo build --target wasm32-unknown-unknown --profile=$PROFILE
cargo ndk --target arm64-v8a build --profile=$PROFILE
cargo ndk --target armeabi-v7a build --profile=$PROFILE
cargo ndk --target x86 build --profile=$PROFILE
cargo ndk --target x86_64 build --profile=$PROFILE
cargo xwin build --target x86_64-pc-windows-msvc --profile=$PROFILE
cargo xwin build --target aarch64-pc-windows-msvc --profile=$PROFILE

mkdir -p dist/shared/linux-x86-64/
mkdir -p dist/shared/linux-aarch64/
mkdir -p dist/shared/win32-x86-64/
mkdir -p dist/shared/win32-aarch64/

cp target/x86_64-unknown-linux-gnu/$PROFILE_PATH/libvodozemac.so dist/shared/linux-x86-64/
cp target/aarch64-unknown-linux-gnu/$PROFILE_PATH/libvodozemac.so dist/shared/linux-aarch64/
cp target/x86_64-pc-windows-msvc/$PROFILE_PATH/vodozemac.dll dist/shared/win32-x86-64/
cp target/aarch64-pc-windows-msvc/$PROFILE_PATH/vodozemac.dll dist/shared/win32-aarch64/

mkdir -p dist/android-shared/x86_64/
mkdir -p dist/android-shared/x86/
mkdir -p dist/android-shared/armeabi-v7a/
mkdir -p dist/android-shared/arm64-v8a/

cp target/x86_64-linux-android/$PROFILE_PATH/libvodozemac.so dist/android-shared/x86_64/
cp target/aarch64-linux-android/$PROFILE_PATH/libvodozemac.so dist/android-shared/arm64-v8a/
cp target/i686-linux-android/$PROFILE_PATH/libvodozemac.so dist/android-shared/x86/
cp target/armv7-linux-androideabi/$PROFILE_PATH/libvodozemac.so dist/android-shared/armeabi-v7a/

mkdir -p dist/static/linux_x64/
mkdir -p dist/static/linux_arm64/
mkdir -p dist/static/mingw_x64/

cp target/x86_64-unknown-linux-gnu/$PROFILE_PATH/libvodozemac.a dist/static/linux_x64/
cp target/aarch64-unknown-linux-gnu/$PROFILE_PATH/libvodozemac.a dist/static/linux_arm64/
cp target/x86_64-pc-windows-msvc/$PROFILE_PATH/vodozemac.lib dist/static/mingw_x64/

mkdir -p dist/web/

(pushd js; npm install; npx rollup -c --configWasm=$WASM_PATH; popd)
cp -a js/dist/ dist/web/
cp js/package.json dist/web

mkdir -p dist/bundles

(pushd dist/shared; zip -r ../bundles/shared.jar .; popd)
(pushd dist/android-shared; zip -r ../bundles/android.jar .; popd)
(pushd dist/static; zip -r ../bundles/static.jar .; popd)
(pushd dist/web; tar cvf ../bundles/web.tar.gz .; popd)
