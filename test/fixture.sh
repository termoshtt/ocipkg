#!/bin/bash
#
# Setup registry for testing. Some tests in ocipkg depends on this setting.
#
set -eu

REGISTRY="localhost:5000"
REPO_NAME="test_repo"

curl -f -X GET http://${REGISTRY}/v2

for tag in tag1 tag2 tag3; do
  docker build -t ${REGISTRY}/${REPO_NAME}:${tag} .
  docker push ${REGISTRY}/${REPO_NAME}:${tag}
done
