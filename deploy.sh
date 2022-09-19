#!/usr/bin/env bash

. .env

set -x

cargo test
cargo build --release

ssh $DEPLOY_HOST sudo systemctl stop $SYSTEMD_SERVICE
rsync -P ./target/release/war-lessons-bot $DEPLOY_HOST:$DEPLOY_PATH
ssh $DEPLOY_HOST sudo systemctl start $SYSTEMD_SERVICE
ssh $DEPLOY_HOST journalctl --no-hostname -S now -qfu $SYSTEMD_SERVICE
