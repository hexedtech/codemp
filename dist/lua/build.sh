#!/bin/sh

cd ../..
cargo build --release --features=lua
mv ./target/release/libcodemp.so ./dist/lua/codemp_lua.so
