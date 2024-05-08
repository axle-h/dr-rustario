#!/bin/bash
set -e

DEST=target/aarch64-unknown-linux-gnu/release

rm -rf $DEST
#mkdir -p $DEST/lib
mkdir -p $DEST
docker build . -t dr-rustario-aarch64 -f Dockerfile.aarch64
docker create --name dr_rustario_aarch64 dr-rustario-aarch64

docker cp dr_rustario_aarch64:/app/target/aarch64-unknown-linux-gnu/release/dr-rustario $DEST
#docker cp dr_rustario_aarch64:/usr/lib/aarch64-linux-gnu/libSDL2_gfx-1.0.so.0.0.2 $DEST/lib/libSDL2_gfx.so
#docker cp dr_rustario_aarch64:/usr/lib/aarch64-linux-gnu/libSDL2_image-2.0.so.0.2.2 $DEST/lib/libSDL2_image.so
#docker cp dr_rustario_aarch64:/usr/lib/aarch64-linux-gnu/libSDL2_mixer-2.0.so.0.2.2 $DEST/lib/libSDL2_mixer.so
#docker cp dr_rustario_aarch64:/usr/lib/aarch64-linux-gnu/libSDL2_ttf-2.0.so.0.14.1 $DEST/lib/libSDL2_ttf.so

docker rm -f dr_rustario_aarch64

scp $DEST/dr-rustario ark@10.0.0.117:/roms/ports/dr-rustario