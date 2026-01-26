#!/usr/bin/env bash
set -euo pipefail

BUMP_STRATEGY="${1:-}"

if [ -z "$BUMP_STRATEGY" ]; then
  echo "Usage: $0 <patch|minor|major>"
  exit 1
fi

if [[ ! "$BUMP_STRATEGY" =~ ^(patch|minor|major)$ ]]; then
  echo "Error: Invalid bump strategy '$BUMP_STRATEGY'"
  echo "Valid options: patch, minor, major"
  exit 1
fi

# Get current version before bump
OLD_VERSION=$(cargo metadata --no-deps --format-version 1 | jq -r '.packages[0].version')

# Bump version across workspace
cargo set-version --workspace --bump "$BUMP_STRATEGY"

# Get new version after bump
NEW_VERSION=$(cargo metadata --no-deps --format-version 1 | jq -r '.packages[0].version')

# Update jpf4826ctl dependency on jpf4826_driver
cd jpf4826ctl
cargo add jpf4826_driver@"$NEW_VERSION" --path ../jpf4826_driver
cd ..

# Update Cargo.lock
cargo update --workspace

# Show result
echo "Version bumped: $OLD_VERSION -> $NEW_VERSION ($BUMP_STRATEGY)"
