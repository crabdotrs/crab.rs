#!/usr/bin/env bash
set -euo pipefail

TAG="${1:-latest}"
IMAGE_NAME="${IMAGE_NAME:-crab}"
PLATFORMS="linux/amd64,linux/arm64"

cd "$(dirname "$0")/../.."

echo "Setting up Docker buildx..."
docker buildx create --use --name crab-builder 2>/dev/null || docker buildx use crab-builder

echo "Building multi-arch image for platforms: $PLATFORMS"
docker buildx build \
  --platform "$PLATFORMS" \
  -t "$IMAGE_NAME:$TAG" \
  --push \
  . 2>/dev/null || \
docker buildx build \
  --platform "$PLATFORMS" \
  -t "$IMAGE_NAME:$TAG" \
  .

echo "Multi-arch build complete."
