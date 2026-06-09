# Copyright 2025 Dotanuki Labs
# SPDX-License-Identifier: MIT

#
# Adapted from :
# https://kerkour.com/rust-docker-from-scratch
# https://labs.iximiuz.com/tutorials/pitfalls-of-from-scratch-images
#
FROM rust:alpine3.21@sha256:7dba3db75b0d5861af2e5dcf5b87347411e84403b5c9835be00ef1e72e299d2a AS builder

RUN apk update && \
    apk upgrade --no-cache && \
    apk add --no-cache lld mold musl musl-dev libc-dev cmake clang clang-dev openssl perl file \
        libressl-dev git build-base bash curl zip gnupg coreutils gcc g++ zstd binutils ca-certificates

WORKDIR /src
COPY . ./
RUN cargo build -p pollux --release


FROM alpine@sha256:4f4ba248d8a2c90a6e52ffdfc194181f7617f9ddaca348d4c550a6b354fc7c2a AS extras

RUN apk update && \
    apk upgrade --no-cache && \
    apk add --no-cache ca-certificates mailcap tzdata

RUN update-ca-certificates

FROM scratch

COPY --from=extras --chmod=444 \
    /etc/passwd \
    /etc/group \
    /etc/nsswitch.conf \
    /etc/mime.types \
    /etc/

COPY --from=extras --chmod=444 /etc/ssl/certs/ca-certificates.crt /etc/ssl/certs/
COPY --from=extras --chmod=444 /usr/share/zoneinfo /usr/share/zoneinfo

COPY --from=builder /src/target/release/pollux /bin/pollux

WORKDIR /tmp

ENTRYPOINT ["/bin/pollux"]
