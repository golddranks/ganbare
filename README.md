# Ganbare
A web service that has something to do with teaching and learning Japanese pronunciation. I license the server code itself as copyleft open source for now, but the contents of the app (example sentences, audio, images etc.) are not licensed, and they are not in this repository. The `static` folder contains some CC 3.0 BY licensed assets that are not by me. As the contents and assets are the "meat" of this app, the usefulness of open sourcing this code is questionable, but then again, why not? If you use this code, kindly inform me too.

## Quickstart using docker-compose

Have Docker installed, and type this in the repo root:
```
$ docker-compose up --build web
```
... then access http://localhost:8080 with your browser.

The Dockerfile is designed to cache the dependencies, so re-builds should be relatively quick.

## Building without Docker

### Build tools
* A working Rust environment (My `rustc -V` prints `rustc 1.46.0 (04488afe3 2020-08-24)`)
* A TypeScript compiler (My `tsc -v` prints `Version 4.0.3`)
* A Sass compiler (My `sass --version` prints `1.26.11`, using the Dart Sass)

### Dependencies not handled by cargo
* OpenSSL (anything supported by Rust crate openssl 0.10)
* libpq (anything supported by Rust crate pq-sys 0.4; try installing PostgreSQL to get this.)

### For poking the database (not hard requirements; the app automigrates)
* For playing with the migrations, you can use the diesel command line tool (`cargo install diesel_cli`, my `diesel --version` prints `diesel 1.4.1`)
* Of course `psql` helps too!

### During build, you need a 256-bit base64-encoded pepper that gets built into the binary:

    $ export GANBARE_BUILDTIME_PEPPER=$(head -c 32 /dev/random | base64)

### All set?

    $ scripts/build_static.sh # Builds TypeScript and SASS and places the results under ./static/
    $ cargo build # Builds the app

## Running without Docker

If you want to run the app locally, it might still help to run the database in Docker:

```
$ docker-compose up db
$ psql -p 5432 -U postgres -h localhost -d ganbare_dev
> Password for user postgres:
< password
psql (12.4, server 13.0 (Debian 13.0-1.pgdg100+1)) 
... yay, seems to work! ^D
```

After configuring (see below), just launch the app:

    $ cargo run

### Configuration

For debug builds, the following directories are used runtime. All files are accessed relative to the directory the app was launched from.

* `static`
* `migrations`
* `templates`
* `audio`
* `images`
* `user_audio`

For release builds, only `static`, `audio`, `user_audio` and `images` are used. (`migrations` and `templates` are compiled statically inside the binary.)

The server is configured using environmental variables, or an `.env` file in the root directory. The following are required:

    GANBARE_DATABASE_URL=postgres://postgres@localhost:5432/ganbare_dev
    GANBARE_RUNTIME_PEPPER 256-bit base64-encoded random value for peppering the password hashes.
    GANBARE_EMAIL_SERVER Whatever e-mail server works for you. e.g. mail.yourisp.net:25, Mailgun, AWS SES...
    GANBARE_SITE_DOMAIN Set this right for cookies etc. to work.
    GANBARE_COOKIE_HMAC_KEY 256-bit base64-encoded random value for signing cookies.

The following have defaults, and you may omit them:

    GANBARE_LOG Log level. Syntax example: ganbare=debug,ganbare_backend=debug. Defaults to debug in debug builds and info in release builds.
    GANBARE_PERF_TRACE prints timings of various operations into debug log. Defaults to to true in debug builds, false in release builds.
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
    GANBARE_PASSWORD_STRETCHING_MS How long new passwords are stretched in milliseconds. Defaults to 500 ms.
    GANBARE_ENABLE_SOURCE_MAPS Defaults to false. Whether it allows to see files in /src using HTTP.
    NAG_EMAIL_ABSENCE_PERIOD_HOURS How much to wait for the user to be absent before sending a nag email. Defaults to 52 hours.
    GANBARE_NAG_EMAIL_GRACE_PERIOD_HOURS How much to wait for the user to ignore the nag email to send another. Defaults to 48 hours.
    GANBARE_EMAIL_EXPIRE_DAYS How old sessions are cleaned. Defaults to 14 days.
    GANBARE_SESSION_EXPIRE_DAYS How account invitation emails are cleaned. Defaults to 14 days.
    GANBARE_TRAINING_PERIOD_DAYS Defaults to 10. This many days since starting training add users to group "posttest".
