#!/bin/bash -euv

case "${TARGET}" in
  web)
    docker run -v $(pwd):/build -w /build ${DOCKER_IMAGE} ci/script_docker.sh
    ;;
  native)
    cargo build --verbose
    cargo test --verbose
    ;;
  *)
    exit 1
    ;;
esac

