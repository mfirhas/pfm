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

### SETTING LOG FILES
echo "setting log files"
PFM_HTTP_STDOUT_LOG_FILE="/var/log/pfm-http.log"
PFM_CRON_STDOUT_LOG_FILE="/var/log/pfm-cron.log"

echo "setting '$PFM_HTTP_STDOUT_LOG_FILE' file..."
if [ -f "$PFM_HTTP_STDOUT_LOG_FILE" ]; then
    echo "File '$PFM_HTTP_STDOUT_LOG_FILE' exists."
else
    echo "File '$PFM_HTTP_STDOUT_LOG_FILE' does not exist. Creating it..."
    sudo touch "$PFM_HTTP_STDOUT_LOG_FILE"
    echo "File '$PFM_HTTP_STDOUT_LOG_FILE' created."
fi

echo "setting '$PFM_CRON_STDOUT_LOG_FILE' file..."
if [ -f "$PFM_CRON_STDOUT_LOG_FILE" ]; then
    echo "File '$PFM_CRON_STDOUT_LOG_FILE' exists."
else
    echo "File '$PFM_CRON_STDOUT_LOG_FILE' does not exist. Creating it..."
    sudo touch "$PFM_CRON_STDOUT_LOG_FILE"
    echo "File '$PFM_CRON_STDOUT_LOG_FILE' created."
fi
sudo chown pfm:pfm $PFM_HTTP_STDOUT_LOG_FILE $PFM_CRON_STDOUT_LOG_FILE
sudo chmod 644 $PFM_HTTP_STDOUT_LOG_FILE $PFM_CRON_STDOUT_LOG_FILE
### END

### SETTING PFM DIRECTORIES
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
### END

echo "Changing owner of $PFM_DATA"
sudo chown pfm:pfm $PFM_DATA
sudo chmod -R 750 $PFM_DATA

EOF

echo "Setup done!"

