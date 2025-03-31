#!/bin/bash

LOCAL_DIR="$1"
REMOTE_USER="$2"
REMOTE_DIR="$3"

ssh $REMOTE_USER@contabo-vps "sudo chown -R $REMOTE_USER:$REMOTE_USER /home/$REMOTE_USER/pfm/pfm-data"

# Rsync command to push the directory (overwriting everything)
rsync -avz --delete "$LOCAL_DIR/" "$REMOTE_USER@contabo-vps:$REMOTE_DIR"

# Check if rsync succeeded
if [ $? -eq 0 ]; then
    echo "✅ Directory successfully pushed to $REMOTE_HOST:$REMOTE_DIR"
else
    echo "❌ Failed to push directory!"
    exit 1
fi

ssh $REMOTE_USER@contabo-vps "sudo chown -R pfm:pfm /home/$REMOTE_USER/pfm/pfm-data"
