FROM rust:1-slim AS builder

WORKDIR /app
COPY Cargo.toml ./
COPY src ./src
COPY tests ./tests
RUN cargo build --release

FROM python:3.12-slim

WORKDIR /app
COPY --from=builder /app/target/release/pstd /usr/local/bin/pstd
COPY python ./python
RUN apt-get update \
    && apt-get install -y --no-install-recommends poppler-utils \
    && rm -rf /var/lib/apt/lists/* \
    && pip install ./python

ENTRYPOINT ["pstd"]
