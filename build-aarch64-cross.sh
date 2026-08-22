#!/bin/bash
set -e

DEST=target/aarch64-unknown-linux-gnu/release

rm -rf $DEST
mkdir -p $DEST
docker build . -t dr-rustario-aarch64 -f Dockerfile.aarch64
docker create --name dr_rustario_aarch64 dr-rustario-aarch64

docker cp dr_rustario_aarch64:/app/target/aarch64-unknown-linux-gnu/release/dr-rustario $DEST

docker rm -f dr_rustario_aarch64

scp $DEST/dr-rustario ark@10.0.0.117:/roms/ports/dr-rustario