#!/bin/bash
#
# Setup registry for testing. Some tests in ocipkg depends on this setting.
#
set -eu

REGISTRY="localhost:5000"
REPO_NAME="test_repo"

SCRIPT_DIR=$(readlink -f $(dirname ${BASH_SOURCE:-$0}))

curl -f -X GET http://${REGISTRY}/v2

for tag in tag1 tag2 tag3; do
  docker build -t ${REGISTRY}/${REPO_NAME}:${tag} ${SCRIPT_DIR}
  docker push ${REGISTRY}/${REPO_NAME}:${tag}
done
