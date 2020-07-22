#!/bin/bash

REMOTE="boats.local"
REMOTE_DIR="/home/frans/Documents/weather"
TARGET="target/armv7-unknown-linux-musleabihf/debug/server"

scp $TARGET $REMOTE:$REMOTE_DIR
ssh $REMOTE 'RUST_BACKTRACE=1 cd /home/frans/Documents/weather && ./server'

