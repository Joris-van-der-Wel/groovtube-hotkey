#!/bin/sh
set -ex
cd "$(dirname "$0")"
rm -f target/groovtube-hotkey-universal
cargo build -r --target=aarch64-apple-darwin
cargo build -r --target=x86_64-apple-darwin
lipo -create -output target/groovtube-hotkey-universal target/aarch64-apple-darwin/release/groovtube-hotkey target/x86_64-apple-darwin/release/groovtube-hotkey
