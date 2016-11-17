FROM clux/muslrust
MAINTAINER Pyry Kontio <pyry.kontio@drasa.eu>

RUN apt-get update && apt-get install -y \
	libpq-dev \
	sqlite3 \
	--no-install-recommends && \
 	rm -rf /var/lib/apt/lists/*

RUN git clone --branch 0.1 https://github.com/golddranks/ganbare.git && cd ganbare
RUN ln -s /lib/x86_64-linux-gnu/libsqlite3.so.0 /lib/x86_64-linux-gnu/libsqlite3.so

ENV RUN DATABASE_URL=postgres://drasa@172.17.0.2/ganbare_testing
ENV GANBARE_DATABASE_URL=postgres://drasa@172.17.0.2/ganbare_testing
ENV GANBARE_BUILDTIME_PEPPER=`cat /dev/urandom | base64 | head -c 42`

RUN cargo install diesel_cli && diesel database setup && cargo build --release
