#!/bin/bash
# Builds the aarch64 PortMaster zip into dist/.
#
#   ./build-portmaster.sh                       -> dist/dr-rustario-vs-rustris.zip
#   DEPLOY_HOST=root@rocknix ./build-portmaster.sh
#       also copies the zip into the device's PortMaster/autoinstall folder; open PortMaster
#       on the device to install it (ROCKNIX default path below, override with DEPLOY_PATH)
set -euo pipefail
cd "$(dirname "$0")"

PORT=dr-rustario-vs-rustris
ARCH=aarch64
IMAGE=$PORT-$ARCH
DIST=dist
STAGE=$DIST/port
BIN=target/aarch64-unknown-linux-gnu/release/$PORT

docker build . -t "$IMAGE" -f Dockerfile.aarch64
container=$(docker create "$IMAGE")
trap 'docker rm -f "$container" > /dev/null' EXIT
mkdir -p "$(dirname "$BIN")"
docker cp "$container:/app/$BIN" "$BIN"

# PortMaster layout: launcher script + port.json at the root, the game in its own folder
rm -rf "$STAGE"
mkdir -p "$STAGE"
cp -r portmaster/. "$STAGE/"
cp "$BIN" "$STAGE/$PORT/$PORT.$ARCH"
chmod +x "$STAGE/$PORT/$PORT.$ARCH" "$STAGE"/*.sh
cp "$STAGE/screenshot.png" "$STAGE/$PORT/screenshot.png"

echo "--- $PORT.$ARCH"
file "$STAGE/$PORT/$PORT.$ARCH"
if command -v aarch64-linux-gnu-readelf > /dev/null; then
  aarch64-linux-gnu-readelf -d "$STAGE/$PORT/$PORT.$ARCH" | grep NEEDED
  echo "max glibc symbol: $(aarch64-linux-gnu-objdump -T "$STAGE/$PORT/$PORT.$ARCH" | grep -o 'GLIBC_[0-9.]*' | sort -uV | tail -1)"
fi

rm -f "$DIST/$PORT.zip"
(cd "$STAGE" && zip -rq "../$PORT.zip" .)
echo "--- $DIST/$PORT.zip"
unzip -l "$DIST/$PORT.zip"

if [ -n "${DEPLOY_HOST:-}" ]; then
  DEPLOY_PATH=${DEPLOY_PATH:-/storage/roms/ports/PortMaster/autoinstall/}
  scp "$DIST/$PORT.zip" "$DEPLOY_HOST:$DEPLOY_PATH"
  echo "copied to $DEPLOY_HOST:$DEPLOY_PATH - open PortMaster on the device to install"
fi
