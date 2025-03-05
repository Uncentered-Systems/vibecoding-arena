#!/bin/bash

# URL for hyperware_process_lib v1.0.3 tarball from crates.io
URL="https://crates.io/api/v1/crates/hyperware_process_lib/1.0.3/download"
TARGET_DIR="utilities/process-lib"
CRATE_NAME="hyperware_process_lib-1.0.3"

# Create target directory if it doesn't exist
mkdir -p "$TARGET_DIR"

# Download the tarball
curl -L "$URL" -o "$CRATE_NAME.tar.gz"

# Extract it
tar -xzf "$CRATE_NAME.tar.gz"

# Move the extracted folder to the target directory
mv "$CRATE_NAME" "$TARGET_DIR/"

# Clean up
rm "$CRATE_NAME.tar.gz"

echo "hyperware_process_lib v1.0.3 installed to $TARGET_DIR/$CRATE_NAME/"