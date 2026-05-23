#!/bin/sh

# Exit immediately if a command exits with a non-zero status
set -e

# --- Configuration & Defaults ---
PROFILE="${1:-release}"  # Defaults to 'release' if not provided
BINARY_NAME="{{crate_name}}"

# Validate profile input
if [ "$PROFILE" != "release" ] && [ "$PROFILE" != "debug" ]; then
    echo "Error: Profile must be 'release' or 'debug'" >&2
    exit 1
fi

# Map profile to cargo flag and target directory structure
if [ "$PROFILE" = "release" ]; then
    CARGO_PROFILE="release"
    TARGET_DIR="release"
else
    CARGO_PROFILE="dev"
    TARGET_DIR="debug"
fi

# --- Target Paths ---
LOCAL_BIN="target/armv7-unknown-linux-gnueabi/$TARGET_DIR/$BINARY_NAME"
REMOTE_HOST="192.168.1.27"
REMOTE_PORT="2222"
REMOTE_USER="reader"
REMOTE_DIR="/mnt/ext1/applications"
SSH_KEY_PATH="$HOME/.ssh/id_rsa"

# --- Execution ---
echo "Setting file descriptor limit..."
ulimit -n 4096

echo "Building with profile: $PROFILE..."
just cargo_profile="$CARGO_PROFILE" build

echo "Adding SSH key..."
eval "$(ssh-agent -s)"
ssh-add "$SSH_KEY_PATH"

echo "Deploying binary to target..."
scp -P "$REMOTE_PORT" -o HostKeyAlgorithms=+ssh-rsa \
    "$LOCAL_BIN" \
    "${REMOTE_USER}@${REMOTE_HOST}:${REMOTE_DIR}/application.app.stage"

echo "Restarting application on target..."
ssh -p "$REMOTE_PORT" -o HostKeyAlgorithms=+ssh-rsa "${REMOTE_USER}@${REMOTE_HOST}" \
  "sh -c \"killall application.app; mv ${REMOTE_DIR}/application.app.stage ${REMOTE_DIR}/application.app; ${REMOTE_DIR}/application.app\""
