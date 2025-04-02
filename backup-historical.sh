#!/bin/bash

REMOTE_USER=$1
REMOTE_PORT=$2
PASSWORD=$3

ssh -p $REMOTE_PORT "$REMOTE_USER@contabo-vps" REMOTE_USER="$REMOTE_USER" PASSWORD="$PASSWORD" 'bash -s' << 'EOF'
  set -e
  
  echo "Current user: $(whoami)"

  DEST_FILE="/home/$REMOTE_USER/files/pfm/historical_backup_$(date +%Y%m%d).zip"
  SOURCE="/home/$REMOTE_USER/pfm/pfm-data/historical"

  echo "source: $SOURCE"
  echo "dest: $DEST_FILE"

  echo "Zipping directory $SOURCE into $DEST_FILE with a password..."
  
  zip -r -P "$PASSWORD" "$DEST_FILE" "$SOURCE"

  sudo chmod 755 $DEST_FILE
 
  echo "Backup created successfully: $DEST_FILE"
EOF
