#!/bin/bash

set -euo pipefail

if [[ $EUID -ne 0 ]]; then
  echo "please run command with sudo"
  exit 1
fi

ARCH_S=$(uname -m)
echo "System detected: $ARCH_S"

REPOSITORY="https://api.github.com/repos/hwisnu222/sarcent/releases/latest"

if ! VERSION=$(curl -sL $REPOSITORY | jq -r ".tag_name"); then
  echo "failed get tag repository"
  exit 1
fi

URL_RELEASE="https://github.com/hwisnu222/sarcent/releases/download/$VERSION/sarcent-$ARCH_S"

echo "Download binary..."
if curl -L $URL_RELEASE -o /usr/local/bin/sarcent; then
  chmod +x /usr/local/bin/sarcent
  echo "Install finished"
else
  echo "failed download binary file"
  exit 1
fi
