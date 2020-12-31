FROM rust:1 as builder

ENV YARA_VERSION=3.7.0
COPY ./ processor-infobserve
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
    cd .. && \
    cargo build

RUN cp /usr/local/lib/libyara.so /usr/lib/libyara.so.3

ENTRYPOINT [ "./target/debug/processor-rs" ]

# FROM ubuntu

# RUN apt update && \
#     apt install gcc libssl -y

# COPY --from=builder /usr/local/bin/yara /usr/local/bin/yara
# COPY --from=builder /usr/local/lib/libyara.so /usr/lib/libyara.so.3
# COPY --from=builder /processor-infobserve/target/debug/processor-rs .

# CMD ["./processor-rs"]