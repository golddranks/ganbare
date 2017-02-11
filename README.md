# Ganbare
A web service that helps me to do what I do. (Protip: Something related to Japanese language and language learning. And science.) I license the server code itself as copyleft open source for now, but the contents of the app (example sentences, audio, images etc.) are not licensed, and they are not in this repository. The `static` folder contains some CC 3.0 BY licensed assets that are not by me. If you use this code, kindly inform me too.

## Requirements to build

* A working nightly Rust environment (`rustc 1.17.0-nightly (c49d10207 2017-02-07)` as of writing this)
* A TypeScript compiler (`tsc -v` prints `Version 2.1.4`)
* A Sass compiler (`sass -v` prints `Sass 3.4.23 (Selective Steve)`)
* Preferably the diesel command line tool installed (`cargo install diesel_cli`)
* Having `psql` installed for poking the database also helps tremendously.

## How to start (1. setup a database 2. create an .env file 3. start the server)

    $ docker run --name ganbare-postgres -d --restart=unless-stopped -e POSTGRES_USER=$USER -e POSTGRES_DB=ganbare_dev -p 127.0.0.1:5432:5432 golddranks/ganbare_database

The server is configured using environmental variables, or an `.env` file in the project directory. The following are required:

    GANBARE_DATABASE_URL=postgres://drasa@localhost/ganbare_dev
    GANBARE_RUNTIME_PEPPER=some 32-byte random value encoded with Base64 (usually 44 ASCII characters) for peppering the password hashes.
    GANBARE_EMAIL_SERVER=mail.yourisp.net:25
    GANBARE_SITE_DOMAIN Set this right for production for cookies etc. to work.

The following have defaults, and you may omit them:

    GANBARE_PARANOID Defaults to true. When on, HTTPS is required. Cookies are sent with "Secure" flag. Strictens the anti-CSRF measures. (Checks Origin & Referer of all mutating HTTP METHODS, and prevents even non-mutating requests to the HTTP API) Enforces ContentSecurityPolicy as an anti-XSS measure.
    GANBARE_EMAIL_SMTP_USERNAME SMTP username. Defaults to empty string.
    GANBARE_EMAIL_SMTP_PASSWORD password. Defaults to empty string.
    GANBARE_SITE_LINK defaults to http://$GANBARE_SITE_DOMAIN:8081. This is used, for example, in email messages to link to the site.
    GANBARE_EMAIL_DOMAIN If the e-mail domain is different than the site domain. (e.g. app in subdomain but emails from main domain) Defaults to $GANBARE_SITE_DOMAIN
    GANBARE_EMAIL_ADDRESS The default e-mail address that the emails from the app are send from. Defaults to support@$GANBARE_EMAIL_DOMAIN
    GANBARE_EMAIL_NAME The name part of the e-mail address. Defaults to empty string.
    GANBARE_SERVER_BINDING defaults to localhost:8080. When running inside a container, change this to 0.0.0.0:8080 for the site to be accessible from host.
    GANBARE_JQUERY defaults to /static/js/jquery.min.js. For production, try https://ajax.googleapis.com/ajax/libs/jquery/3.1.1/jquery.min.js
    GANBARE_FONT_URL defaults to /static/fonts/default.css. For production, try https://fonts.googleapis.com/css?family=Source+Sans+Pro:300
    GANBARE_USER_AUDIO_DIR defaults to "user_audio" (dir relative to app working directory) You shouldn't need to change this, but it's possible.
    GANBARE_AUDIO_DIR defaults to "audio" (dir relative to app working directory) You shouldn't need to change this, but it's possible.
    GANBARE_IMAGES_DIR defaults to "images" (dir relative to app working directory) You shouldn't need to change this, but it's possible.
    GANBARE_CONTENT_SECURITY_POLICY Sets the contents of Content-Security-Policy header. Defaults to "default-src 'self'; style-src 'self' 'unsafe-inline' https://fonts.googleapis.com; font-src 'self' https://fonts.gstatic.com https://fonts.googleapis.com; script-src 'self' 'unsafe-inline' https://ajax.googleapis.com"
    GANBARE_CACHE_MAX_AGE Sets the max-age of cache control of static files. Defaults to conservative 30 seconds. Change this to a larger number on production!
    GANBARE_SERVER_THREADS Sets the amount of threads. Defaults to 20. Note that the server is syncronous at the moment, so recommended setting for production are: HAProxy with option http-server-close and server maxconns set to the same value as GANBARE_SERVER_THREADS.
    GANBARE_PERF_TRACE prints timings of various operations into debug log. Defaults to false.
    RUST_LOG Log level. Try ganbare=debug,ganbare_backend=debug if you want to debug stuff.

During build, you need the following env var too: 

    GANBARE_BUILDTIME_PEPPER=some 32-byte random value encoded with Base64 (usually 44 ASCII characters) for peppering the password hashes.

After creating an `.env` file, start the server: (`./build_everything.sh` ensures that not only Rust but TypeScript and SCSS files are built too.)

    $ ./build_everything.sh && cargo run

Navigate to localhost:8080 with your browser. For debug builds, directories `static`, `migrations` and `templates`, `audio` and `images` are used runtime.
For release builds, only `static`, `audio` and `images` are used, as `migrations` and `templates` are compiled statically inside the binary.

## How to build & deploy easily
Just do

    $ ./build_musl && ./deploy_binary_testing

## How to build a distributable, statically linked MUSL binary

The `Dockerfile.build` is for that. It assembles the build environment for building a statically-linked MUSL-based Linux binary:

    $ docker build -f Dockerfile.build -t golddranks/ganbare_build .
    $ docker run -it --rm --name ganbare_builder -v $PWD:/workdir golddranks/ganbare_build

There you go, a readymade binary in your target folder.

## How to build and run a container for distribution.

`Dockerfile.run` is for that.

    $ docker build -f Dockerfile.run -t golddranks/ganbare_run .
    $ docker run -d --name ganbare_runner -p 8080:8080 -e "GANBARE_RUNTIME_PEPPER=${GANBARE_RUNTIME_PEPPER}" -v $PWD/audio:/ganbare/audio -v $PWD/images:/ganbare/images golddranks/ganbare_run

Notes about running: Ganbare needs a PostgreSQL database connection, and mounted volume for user-uploaded images and audio.
Mount the volume using docker `-v` flag. Everthing else you shall configure using environmental variables.
