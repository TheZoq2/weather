#!/bin/bash

REMOTE="boats.local"
REMOTE_DIR="/home/frans/Documents/weather"
TARGET="target/armv7-unknown-linux-musleabihf/debug/server"
FRONTEND_TARGET="frontend/output/index.html"
FRONTEND_DEST="${REMOTE_DIR}/frontend/output/index.html"

echo "Building frontend"
make -C frontend

echo "Copying server exe"
rsync $TARGET $REMOTE:$REMOTE_DIR
echo "Copying frontend"
rsync $FRONTEND_TARGET $REMOTE:$FRONTEND_DEST
ssh -t $REMOTE 'RUST_BACKTRACE=1 cd /home/frans/Documents/weather && ./server'

