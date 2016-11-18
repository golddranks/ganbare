FROM debian:jessie
MAINTAINER Pyry Kontio <pyry.kontio@drasa.eu>

RUN apt-get update && apt-get install -y \
  curl \
  git \
  libpq-dev \
  sqlite3 \
  ca-certificates \
  g++ \
  postgresql \ 
  postgresql-client \
  --no-install-recommends && \
  rm -rf /var/lib/apt/lists/*

RUN curl https://sh.rustup.rs -sSf | sh -s -- -y --default-toolchain nightly-2016-11-06
RUN git clone --branch 0.1.1 https://github.com/golddranks/ganbare.git
RUN ln -s /usr/lib/x86_64-linux-gnu/libsqlite3.so.0 /usr/lib/x86_64-linux-gnu/libsqlite3.so
RUN /root/.cargo/bin/cargo install diesel_cli

ENV DATABASE_URL=postgres://root@%2Fvar%2Frun%2Fpostgresql/ganbare_build
ENV GANBARE_DATABASE_URL=postgres://root@%2Fvar%2Frun%2Fpostgresql/ganbare_build

WORKDIR ganbare

RUN /etc/init.d/postgresql start && \
	su - postgres -c "createuser root; psql -c 'alter user root with createdb'" && \
	bash -l -c "/root/.cargo/bin/diesel database setup;"

RUN /etc/init.d/postgresql start && \
	bash -l -c "GANBARE_BUILDTIME_PEPPER=`cat /dev/urandom | head -c 32 | base64` \
				  /root/.cargo/bin/cargo build --release"
