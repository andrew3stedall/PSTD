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
RUN pip install ./python

ENTRYPOINT ["pstd"]
