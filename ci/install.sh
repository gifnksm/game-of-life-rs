#!/bin/bash -eu

case "${TARGET}" in
  web)
    git clone https://github.com/juj/emsdk.git
    . emsdk/emsdk_env.sh
    emsdk install -j1 sdk-1.37.9-64bit binaryen-tag-1.37.9-64bit
    ;;
  native)
    ;;
  *)
    exit 1
    ;;
esac

