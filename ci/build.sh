#!/bin/bash -euv

case "${TARGET}" in
  web)
    . emsdk/emsdk_env.sh
    make
    ;;
  native)
    cargo build --verbose
    cargo test --verbose
    ;;
  *)
    exit 1
    ;;
esac

