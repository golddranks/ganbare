FROM rust:1.46-alpine

RUN apk add --no-cache musl-dev openssl-dev postgresql-dev

WORKDIR /work
RUN mkdir -p src && mkdir -p ganbare_backend/src

# Pre-build and cache the dependencies
COPY Cargo.toml Cargo.lock ./
COPY ganbare_backend/Cargo.toml ganbare_backend/Cargo.lock ./ganbare_backend/
RUN echo "fn main() {}" > src/main.rs && touch -t 197001010000 src/main.rs && touch -t 197001010000 ganbare_backend/src/lib.rs
RUN cargo build --release

COPY . .
ARG GANBARE_BUILDTIME_PEPPER
RUN cargo build --release

FROM node:10-alpine
RUN apk add --no-cache sassc

WORKDIR /work
COPY static static

COPY src/sass src/sass
RUN sassc src/sass/main.scss static/css/main.css

COPY tsconfig.json .
COPY src/ts src/ts
RUN npm install -g typescript
RUN tsc

FROM alpine:latest
WORKDIR /root
COPY --from=0 /work/target/release/ganbare .
COPY --from=0 /work/static ./static
CMD ["/root/ganbare"]
