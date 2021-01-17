FROM rust:1 as builder

ENV YARA_VERSION=3.7.0
WORKDIR /processor-infobserve

RUN apt update && \
    apt install wget automake libtool make gcc build-essential pkg-config llvm clang -y && \
    wget https://github.com/VirusTotal/yara/archive/v${YARA_VERSION}.tar.gz && \
    tar -zxf v${YARA_VERSION}.tar.gz && \
    cd yara-${YARA_VERSION} && \
    ./bootstrap.sh && \
    ./configure && \
    make && \
    make install && \
    cp /usr/local/lib/libyara.so /usr/lib/libyara.so.3

COPY ./ .
RUN ls && cargo build

ENTRYPOINT [ "./target/debug/processor-rs" ]