#!/usr/bin/env bash
set -euo pipefail

TAG="${TAG:-latest}"
IMAGE_NAME="${IMAGE_NAME:-crab}"

cd "$(dirname "$0")/../.."

echo "Starting shell in crab container..."
docker run --rm -it \
  -v "$(pwd):/workspace" \
  -w /workspace \
  --entrypoint /bin/bash \
  "$IMAGE_NAME:$TAG"
