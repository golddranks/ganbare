# Ganbare
A web service that helps me to do what I do. (Protip: Something related to Japanese language and language learning. And science.)

## How to start (1. setup a database 2. set an admin account 3. create an .env file 4. start the server)

    $ docker run --name ganbare-postgres -d --restart=unless-stopped -e POSTGRES_USER=$USER -p 127.0.0.1:5432:5432 postgres
    $ diesel setup

An "admin" account is created automatically. You have to set the password for that account using the `user` CLI tool found in the bin folder.

    $ cargo run --bin user -- passwd admin

The server is configured using environmental variables, or an `.env` file in the project directory:

    GANBARE_DATABASE_URL=postgres://drasa@localhost/ganbare_dev
    GANBARE_RUNTIME_PEPPER=some 32-byte random value encoded with Base64 (usually 44 ASCII characters) for peppering the password hashes.
    GANBARE_SERVER_BINDING=0.0.0.0:8080
    GANBARE_SITE_DOMAIN=testing.ganba.re
    GANBARE_EMAIL_DOMAIN=testing.ganba.re

After creating an `.env` file, start the server:

    $ cargo run --bin server

Navigate to localhost:8080 with your browser.


## How to build a distributable binary

The `Dockerfile.build` is for that. It assembles the build environment for building a statically-linked MUSL-based Linux binary:

    $ docker build -f Dockerfile.build -t golddranks/ganbare_build .
    $ docker run -it --rm --name ganbare_builder -v $PWD:/ganbare golddranks/ganbare_build

There you go, a readymade binary in your target folder.

## How to build and run a container for distribution.

`Dockerfile.run` is for that.

    $ docker build -f Dockerfile.run -t golddranks/ganbare_run .
    $ docker run -d --name ganbare_runner -p 8080:8080 golddranks/ganbare_run

Notes about running: Ganbare needs a PostgreSQL database connection, and mounted volume for user-uploaded images and audio.
Mount the volume using docker `-v` flag. Everthing else you shall configure using environmental variables.

## Notes

    (future additions for conf)
    GANBARE_JQUERY for specifying jquery URL
    GANBARE_MOUNT_DIR for specifying the mounted volume dir for user-uploaded stuff
    GANBARE_EMAIL_SERVER for outbound email
    GANBARE_ASSET_DIR if not set, SQL migrations, HTML templates and assets are compiled into the binary
