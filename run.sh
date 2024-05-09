#!/bin/bash

# Check if an argument is provided for the environment file path
if [ -z "$1" ]; then
    ENV_FILE=".env"
else
    ENV_FILE="$1"
fi

# Load environment variables
source "$ENV_FILE"

# Build the project
cargo build

# Run the server
cargo run -- \
  --host $OUT_DIR \
  --port $SERVER_URL