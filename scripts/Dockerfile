FROM rust:1.46-alpine

RUN apk add --no-cache musl-dev openssl-dev postgresql-dev

WORKDIR /work
RUN mkdir -p src && mkdir -p ganbare_backend/src

# Pre-build and cache the dependencies
COPY Cargo.toml Cargo.lock /work/
COPY ganbare_backend/Cargo.toml ganbare_backend/Cargo.lock /work/ganbare_backend/
RUN echo "fn main() {}" > src/main.rs && touch ganbare_backend/src/lib.rs
RUN cargo build --release

COPY . .
ARG GANBARE_BUILDTIME_PEPPER
RUN touch src/main.rs && touch ganbare_backend/src/lib.rs && cargo build --release

FROM alpine:latest
WORKDIR /root
COPY --from=0 /work/target/release/ganbare .
CMD ["/root/ganbare"]
