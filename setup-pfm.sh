#!/bin/bash

## this script will setup env for pfm and its data

USERNAME="$1"
REMOTE_PORT="$2"
NEW_PATH="/home/$USERNAME/pfm/new/"
ROLLBACK_PATH="/home/$USERNAME/pfm/rollback/"
PFM_DATA="/home/$USERNAME/pfm/pfm-data/"

ssh -P $REMOTE_PORT "$USERNAME@contabo-vps" USERNAME="$USERNAME" NEW_PATH="$NEW_PATH" ROLLBACK_PATH="$ROLLBACK_PATH" PFM_DATA="$PFM_DATA" 'bash -s' << 'EOF'
set -e

echo "Current user: $(whoami)"

if [ ! -d "$NEW_PATH" ]; then
    echo "📁 Directory $NEW_PATH does not exist. Creating: $NEW_PATH"
    mkdir -p "$NEW_PATH"
    echo "✅ Directory created successfully."
else
    echo "✅ Directory already exists: $NEW_PATH"
fi

if [ ! -d "$ROLLBACK_PATH" ]; then
    echo "📁 Directory $ROLLBACK_PATH does not exist. Creating: $ROLLBACK_PATH"
    mkdir -p "$ROLLBACK_PATH"
    echo "✅ Directory created successfully."
else
    echo "✅ Directory already exists: $ROLLBACK_PATH"
fi

if [ ! -d "$PFM_DATA" ]; then
    echo "📁 Directory $PFM_DATA does not exist. Creating: $PFM_DATA"
    mkdir -p "$PFM_DATA"
    echo "✅ Directory created successfully."
else
    echo "✅ Directory already exists: $PFM_DATA"
fi

echo "Changing owner of $PFM_DATA"
sudo chown pfm:pfm $PFM_DATA
sudo chmod -R 750 $PFM_DATA

EOF

echo "Setup done!"

