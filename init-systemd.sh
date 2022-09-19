#!/usr/bin/env bash

. .env
set -eux

sudo tee /etc/systemd/system/$SYSTEMD_SERVICE.service << EOF
[Unit]
Description=WarLessons telegram bot
Conflicts=shutdown.target

[Service]
User=$USER
WorkingDirectory=$PWD
ExecStart=$PWD/war-lessons-bot
Restart=always
RestartSec=5

[Install]
WantedBy=multi-user.target
EOF

sudo systemctl enable $SYSTEMD_SERVICE
sudo systemctl start $SYSTEMD_SERVICE
