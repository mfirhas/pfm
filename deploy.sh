#!/bin/bash

set -e

# VPS username
USERNAME="$1"
# VPS SSH port
REMOTE_PORT="$2"
# local path to release binary
LOCAL_BINARY_PATH="$3"
# the name of binary produced by compilation
BINARY_NAME=$(basename "$LOCAL_BINARY_PATH")
# path to .env file
LOCAL_ENV_FILE="$4"
# path to api_keys.json file containing api keys for client
LOCAL_API_KEYS_FILE="$5"
# remote path for root pfm
PFM_PATH="/home/$USERNAME/pfm"
# remote path to be executed from systemd
BINARY_PATH="$PFM_PATH/$BINARY_NAME"
# remote path to new binary before moved into BINARY_PATH
NEW_PATH="$PFM_PATH/new"
NEW_BINARY_PATH="$NEW_PATH/$BINARY_NAME"
# remote path to old binary before new deployment
ROLLBACK_BINARY_PATH="$PFM_PATH/rollback/$BINARY_NAME"

echo "Deploying $LOCAL_BINARY_PATH into remote $NEW_BINARY_PATH..."
echo "-------------------------------------------------------------"

echo "Copying $LOCAL_BINARY_PATH to remote $NEW_BINARY_PATH..."
scp -P $REMOTE_PORT $LOCAL_BINARY_PATH $USERNAME@contabo-vps:$NEW_PATH
scp -P $REMOTE_PORT $LOCAL_ENV_FILE $USERNAME@contabo-vps:$PFM_PATH
scp -P $REMOTE_PORT $LOCAL_API_KEYS_FILE $USERNAME@contabo-vps:$PFM_PATH

echo "Updating binary on remote..."
ssh "$USERNAME@contabo-vps" USERNAME="$USERNAME" BINARY_NAME="$BINARY_NAME" BINARY_PATH="$BINARY_PATH" NEW_BINARY_PATH="$NEW_BINARY_PATH" ROLLBACK_BINARY_PATH="$ROLLBACK_BINARY_PATH" 'bash -s' << 'EOF'
set -e

echo "Current user: $(whoami)"

if [ -f "$BINARY_PATH" ]; then
    echo "Moving current binary to rollback path"
    mv $BINARY_PATH $ROLLBACK_BINARY_PATH
fi

echo "Moving new binary into current binary"
mv $NEW_BINARY_PATH $BINARY_PATH

echo "Change owner to pfm"
sudo chown pfm:pfm $BINARY_PATH

echo "Set executable of new binary"
sudo chmod +x $BINARY_PATH

echo "Binary $BINARY_NAME updated successfully!"

echo "Restarting service..."
sudo systemctl restart $BINARY_NAME

echo "$BINARY_NAME status:"
sudo systemctl status $BINARY_NAME
EOF

echo "Successfully deployed"
