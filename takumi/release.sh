#!/bin/bash

set -e

VERSION=$(jq -r '.version' package.json)
echo "Checking version: $VERSION"

if curl -f https://crates.io/api/v1/crates/takumi/$VERSION 2>&1; then
    echo "Version $VERSION already exists on crates.io. Skipping publish."
else
    echo "Version $VERSION not found. Proceeding with publish..."
    cargo publish
fi
