FROM rust:1.74.1-buster as builder
WORKDIR /app

RUN apt update && apt install lld clang -y

COPY ./Cargo.toml ./Cargo.toml
COPY ./Cargo.lock ./Cargo.lock

COPY ./src ./src

RUN cargo install --path .

FROM debian:bullseye-slim AS runtime

WORKDIR /app

RUN apt-get update -y \
    && apt-get install -y --no-install-recommends openssl ca-certificates \
    && apt-get autoremove -y \
    && apt-get clean -y \
    && rm -rf /var/lib/apt/lists/*

COPY --from=builder /app/target/release/uploader uploader

COPY ./assets ./assets
COPY ./views ./views
COPY ./frontend ./frontend

ENV APP_ENVIRONMENT=production

EXPOSE 8000

CMD ["./uploader"]
