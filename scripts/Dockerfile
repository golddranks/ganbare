FROM registry.gitlab.com/rust_musl_docker/image:stable-1.46.0

WORKDIR /work
RUN mkdir -p src && mkdir -p ganbare_backend/src

# Pre-build and cache the dependencies
COPY Cargo.toml Cargo.lock ./
COPY ganbare_backend/Cargo.toml ganbare_backend/Cargo.lock ./ganbare_backend/
RUN echo "fn main() {}" > src/main.rs && touch ganbare_backend/src/lib.rs
RUN cargo build -v --release --target=x86_64-unknown-linux-musl

COPY ganbare_backend ./ganbare_backend
COPY migrations ./migrations
ARG GANBARE_BUILDTIME_PEPPER
RUN touch ganbare_backend/src/lib.rs && cargo build -v --release --target=x86_64-unknown-linux-musl

COPY src ./src
COPY templates ./templates
RUN touch src/main.rs && cargo build -v --release --target=x86_64-unknown-linux-musl



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



FROM alpine
WORKDIR /srv
COPY --from=0 /work/target/x86_64-unknown-linux-musl/release/ganbare .
COPY --from=0 /etc/ssl/certs/ca-certificates.crt ./ca-certificates.ctr
COPY --from=1 /work/static ./static

VOLUME /srv/audio /srv/images /srv/user_audio
ENV GANBARE_SERVER_BINDING=0.0.0.0:8080 \
	GANBARE_JQUERY=https://ajax.googleapis.com/ajax/libs/jquery/3.1.1/jquery.min.js \
	GANBARE_FONT_URL=https://fonts.googleapis.com/css?family=Source+Sans+Pro:300 \
	GANBARE_EMAIL_NAME=ganba.re応援団 \
	SSL_CERT_FILE=/srv/ca-certificates.crt

EXPOSE 8080
CMD ["/srv/ganbare"]
