#!/usr/bin/env bash
set -euo pipefail

TAG="${1:-latest}"
REGISTRY="${REGISTRY:-ghcr.io}"
IMAGE_NAME="${IMAGE_NAME:-crab}"
ORG="${ORG:-crabdotrs}"

echo "Tagging for registry..."
docker tag "$IMAGE_NAME:$TAG" "$REGISTRY/$ORG/$IMAGE_NAME:$TAG"

echo "Pushing to $REGISTRY/$ORG/$IMAGE_NAME:$TAG"
docker push "$REGISTRY/$ORG/$IMAGE_NAME:$TAG"

echo "Push complete."
