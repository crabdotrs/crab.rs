#!/usr/bin/env bash
set -euo pipefail

TAG="${TAG:-latest}"
IMAGE_NAME="${IMAGE_NAME:-crab}"

cd "$(dirname "$0")/../.."

echo "Running crab in Docker..."
docker run --rm -it \
  -v "$(pwd):/workspace" \
  -w /workspace \
  "$IMAGE_NAME:$TAG" \
  "$@"
