#!/bin/bash

set -e

USERNAME=$1
REMOTE_PORT=$2
CONFIG_FILE=$3
PFM_PATH="/home/$USERNAME/pfm"

echo "Copying '$CONFIG_FILE' to remote '$PFM_PATH'"
scp -P $REMOTE_PORT $CONFIG_FILE $USERNAME@contabo-vps:$PFM_PATH

echo "Restarting services..."
ssh "$USERNAME@contabo-vps" 'bash -s' << 'EOF'
set -e

echo "Current user: $(whoami)"

echo "Restarting pfm-http..."
sudo systemctl restart pfm-http

sudo systemctl status pfm-http

echo "Restarting pfm-cron..."
sudo systemctl restart pfm-cron

sudo systemctl status pfm-cron

EOF

echo "Successfully update config"
