#!/usr/bin/env bash
set -euo pipefail

TAG="${TAG:-latest}"
IMAGE_NAME="${IMAGE_NAME:-crab}"

cd "$(dirname "$0")/../.."

echo "Running tests in Docker container..."
docker run --rm \
  -v "$(pwd):/workspace" \
  -w /workspace/crab.rs \
  --entrypoint cargo \
  "$IMAGE_NAME:$TAG" \
  test --workspace

echo "Docker tests passed."
