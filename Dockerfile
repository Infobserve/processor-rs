FROM rust:1 as builder

WORKDIR /processor-infobserve

RUN apt update && \
    apt install wget automake libtool make gcc build-essential pkg-config llvm clang -y

COPY ./ .
RUN ls && cargo build

ENTRYPOINT [ "./target/debug/processor-rs" ]