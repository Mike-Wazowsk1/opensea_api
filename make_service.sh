#!/bin/bash

echo "[Unit]
Description=Bitgesell lotto service
After=network.target

[Service]
User=root
Type=simple
WorkingDirectory=$PWD
ExecStart=$PWD/target/release/opensea_api
Restart=always

[Install]
WantedBy=multi-user.target" >/etc/systemd/system/lotto.service
echo "Service build and moved"
systemctl enable lotto.service
echo "Service enabled for start at boot"
systemctl start lotto.service
echo "Service started"
