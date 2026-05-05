#!/usr/bin/env bash
set -euo pipefail

TAG="${TAG:-latest}"
IMAGE_NAME="${IMAGE_NAME:-crab}"
PROJECT_DIR="${PROJECT_DIR:-$(pwd)}"

cd "$(dirname "$0")/../.."

echo "Starting development container with hot reload..."
docker run --rm -it \
  -v "$PROJECT_DIR:/workspace" \
  -w /workspace \
  -p 8080:8080 \
  -e RUST_LOG=debug \
  -e RUST_BACKTRACE=1 \
  --entrypoint /bin/bash \
  "$IMAGE_NAME:$TAG" \
  -c "cd /workspace && exec /bin/bash"
