#!/bin/bash -eu

case "${TARGET}" in
  web)
    docker build --rm -t ${DOCKER_IMAGE} \
      --build-arg RUST_VERSION=${TRAVIS_RUST_VERSION} \
      --build-arg BUILD_UID=$(id -u) \
      --build-arg BUILD_GID=$(id -g) \
      ci
    ;;
  native)
    ;;
  *)
    exit 1
    ;;
esac

