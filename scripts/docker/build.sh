#!/usr/bin/env bash
set -euo pipefail

TAG="${1:-latest}"
IMAGE_NAME="${IMAGE_NAME:-crab}"

cd "$(dirname "$0")/../.."

echo "Building Docker image: $IMAGE_NAME:$TAG"
docker build -t "$IMAGE_NAME:$TAG" .

echo "Build complete."
echo "Run with: docker run -v \$(pwd):/workspace $IMAGE_NAME:$TAG <command>"
