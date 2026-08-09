# Build stage
FROM rust:1-alpine AS builder

RUN apk add --no-cache musl-dev openssl-dev openssl-libs-static pkgconfig git

WORKDIR /app

COPY . .

RUN git submodule update --init --recursive 2>/dev/null || true

# Build the tracker client binaries directly in the submodule
RUN cd torrust-tracker && \
    RUSTFLAGS="--allow unexpected_cfgs" cargo build --release \
    --bin udp_tracker_client \
    --bin http_tracker_client

# Build your own project
RUN cargo build --release

# Runtime stage
FROM alpine:latest

RUN apk add --no-cache ca-certificates

COPY --from=builder /app/target/release/magnetz /usr/local/bin/magnetz
COPY --from=builder /app/torrust-tracker/target/release/udp_tracker_client /usr/local/bin/udp_tracker_client
COPY --from=builder /app/torrust-tracker/target/release/http_tracker_client /usr/local/bin/http_tracker_client

CMD ["magnetz"]
