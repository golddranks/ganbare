FROM alpine:3.12

WORKDIR /work
RUN apk add --no-cache musl-dev curl make perl linux-headers gcc
RUN curl -sL http://www.openssl.org/source/openssl-1.1.1h.tar.gz | tar xz --strip-components 1
RUN ./Configure no-shared no-async linux-x86_64 && make



FROM rust:1.46-alpine

RUN apk add --no-cache musl-dev 'postgresql-dev<12.0' openssl-dev openssl-libs-static bash # bash is temporary to allow longer commands in shell

ENV PQ_LIB_STATIC_X86_64_UNKNOWN_LINUX_MUSL=true \
    OPENSSL_STATIC=true \
    LIBZ_SYS_STATIC=1

ENV RUSTFLAGS='-C target-feature=-crt-static'

WORKDIR /work
RUN mkdir -p src && mkdir -p ganbare_backend/src

COPY test ./test

# Pre-build and cache the dependencies
COPY Cargo.toml Cargo.lock ./
COPY ganbare_backend/Cargo.toml ganbare_backend/Cargo.lock ./ganbare_backend/
RUN echo "fn main() {}" > src/main.rs && touch ganbare_backend/src/lib.rs
RUN cargo build --release
RUN rm ./target/release/deps/libganbare*
RUN rm ./target/release/ganbare

COPY src ./src
COPY ganbare_backend ./ganbare_backend
COPY migrations ./migrations
COPY templates ./templates
COPY posttest.rs pretest.rs ./
ARG GANBARE_BUILDTIME_PEPPER
RUN cargo build --release --target=x86_64-unknown-linux-musl



FROM node:14-alpine3.12
RUN apk add --no-cache sassc

WORKDIR /work
COPY static ./static

COPY src/sass ./src/sass
RUN sassc src/sass/main.scss static/css/main.css

COPY tsconfig.json .
COPY src/ts ./src/ts
RUN npm install -g typescript
RUN tsc



FROM alpine:3.12
WORKDIR /root
COPY --from=1 /work/target/release/ganbare .
COPY --from=2 /work/static ./static
CMD ["/root/ganbare"]
