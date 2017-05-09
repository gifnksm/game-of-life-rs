FROM ubuntu:xenial
MAINTAINER NAKASHIMA, Makoto <makoto.nksm@gmail.com>

ARG RUST_VERSION
ARG BUILD_UID
ARG BUILD_GID

ENTRYPOINT ["/bin/bash"]
RUN echo "deb http://apt.llvm.org/xenial/ llvm-toolchain-xenial-3.9 main" >> /etc/apt/sources.list
RUN echo "deb-src http://apt.llvm.org/xenial/ llvm-toolchain-xenial-3.9 main" >> /etc/apt/sources.list
RUN apt-key adv --keyserver keyserver.ubuntu.com --recv-keys 15CF4D18AF4F7421
RUN apt-get -y update
RUN apt-get -y upgrade
RUN apt-get -y install build-essential git python curl libclang1-3.9

RUN groupadd --gid ${BUILD_GID} build
RUN useradd --uid ${BUILD_UID} --gid ${BUILD_GID} --create-home build
RUN mkdir /emsdk
RUN chown build:build /emsdk
USER build

RUN git clone https://github.com/juj/emsdk.git /emsdk
RUN /emsdk/emsdk install -j1 latest
RUN /emsdk/emsdk activate latest

RUN curl -sSf https://build.travis-ci.org/files/rustup-init.sh | sh -s -- --default-toolchain=$RUST_VERSION -y
ENV PATH=/home/build/.cargo/bin:${PATH}
RUN rustup target add asmjs-unknown-emscripten
