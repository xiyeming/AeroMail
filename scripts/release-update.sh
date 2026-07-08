#!/bin/bash

# Release update script for AeroMail
# This script helps create a GitHub release with update artifacts

set -e

VERSION=${1:-"0.0.23"}
PRIVATE_KEY=${TAURI_SIGNING_PRIVATE_KEY:-"/tmp/private.pem"}

if [ ! -f "$PRIVATE_KEY" ]; then
  echo "Error: Private key not found at $PRIVATE_KEY"
  echo "Set TAURI_SIGNING_PRIVATE_KEY environment variable"
  exit 1
fi

echo "Creating release for version $VERSION..."

# Check if release exists
if gh release view "v$VERSION" > /dev/null 2>&1; then
  echo "Release v$VERSION already exists"
  exit 1
fi

# Create release with assets
gh release create "v$VERSION" \
  --title "v$VERSION" \
  --notes "Release v$VERSION" \
  --latest

echo "Release v$VERSION created successfully!"
echo ""
echo "Next steps:"
echo "1. Build the app with: TAURI_SIGNING_PRIVATE_KEY=$PRIVATE_KEY cargo tauri build"
echo "2. Upload the generated artifacts to the release"
echo "3. Update latest.json with the correct signatures"
