#!/bin/bash

# Script to connect to Digital Ocean droplet and run docker ps
# Password is embedded for convenience

DROPLET_ID="523262403"
DROPLET_PASSWORD="vFwSQaGv23H16MnW"

# Check if sshpass is installed
if ! command -v sshpass &> /dev/null; then
    echo "Error: sshpass is not installed. Install it with: brew install sshpass"
    exit 1
fi

# Get droplet IP address
echo "Getting droplet IP address..."
DROPLET_IP=$(doctl compute droplet get $DROPLET_ID --format PublicIPv4 --no-header)

if [ -z "$DROPLET_IP" ]; then
    echo "Error: Could not get droplet IP. Make sure doctl is authenticated: doctl auth init"
    exit 1
fi

echo "Connecting to droplet $DROPLET_ID at $DROPLET_IP..."
echo "Running docker ps, then opening interactive shell..."
echo ""
sshpass -p "$DROPLET_PASSWORD" ssh -t -o StrictHostKeyChecking=no root@$DROPLET_IP "docker ps; echo ''; echo 'Interactive shell ready. Type exit to disconnect.'; exec bash"
