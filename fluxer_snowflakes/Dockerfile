# SPDX-License-Identifier: AGPL-3.0-or-later

FROM rust:1-bookworm AS builder

WORKDIR /usr/src/app

COPY . .

RUN cargo build --release -p fluxer-snowflakes

FROM debian:bookworm-slim

ARG BUILD_VERSION=""

WORKDIR /usr/local/bin

RUN apt-get update && apt-get install -y --no-install-recommends ca-certificates && \
	rm -rf /var/lib/apt/lists/*

COPY --from=builder /usr/src/app/target/release/fluxer-snowflakes /usr/local/bin/fluxer-snowflakes

ENV BUILD_VERSION="${BUILD_VERSION}"

USER 65532:65532

EXPOSE 8090

CMD ["/usr/local/bin/fluxer-snowflakes"]
