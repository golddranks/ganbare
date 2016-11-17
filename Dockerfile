FROM liuchong/rustup:nightly
MAINTAINER Pyry Kontio <pyry.kontio@drasa.eu>

RUN rustup install nightly-2016-11-06

RUN rustup default nightly-2016-11-06

RUN apt-get update && \
	apt-get install -y git

WORKDIR /opt

RUN git clone https://github.com/golddranks/ganbare.git

WORKDIR /opt/ganbare

RUN cargo build --release

ENV GANBARE_DATABASE_URL=postgres://drasa@localhost/ganbare_testing
ENV GANBARE_BUILDTIME_PEPPER=4l1S1zMc0sqltaga/plxvzHcq0z+gQI5n7DL53jjy9E=
ENV GANBARE_RUNTIME_PEPPER=4Y23isyrgd9ML/jJonemXNL0PnbMyerqnejHPBnDLaY=
ENV GANBARE_SERVER_BINDING=localhost:8080
ENV GANBARE_SITE_DOMAIN=testing.ganba.re
ENV GANBARE_EMAIL_DOMAIN=testing.ganba.re

EXPOSE 8080
