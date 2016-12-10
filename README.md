# Ganbare
A web service that helps me to do what I do. (Protip: Something related to Japanese language and language learning. And science.) I license the server code itself as copyleft open source for now, but the contents of the app (example sentences, audio, images etc.) are not licensed, and they are not in this repository. The `static` folder contains some CC 3.0 BY licensed assets that are not by me. If you use this code, kindly inform me too.

## How to start (1. setup a database 2. create an .env file 3. start the server)

    $ docker run --name ganbare-postgres -d --restart=unless-stopped -e POSTGRES_USER=$USER -e POSTGRES_DB=ganbare_dev -p 127.0.0.1:5432:5432 golddranks/ganbare_database

The server is configured using environmental variables, or an `.env` file in the project directory. The following are required:

    GANBARE_DATABASE_URL=postgres://drasa@localhost/ganbare_dev
    GANBARE_RUNTIME_PEPPER=some 32-byte random value encoded with Base64 (usually 44 ASCII characters) for peppering the password hashes.
    GANBARE_EMAIL_SERVER=mail.yourisp.net:25

The following have defaults, and you may omit them:

    GANBARE_SITE_DOMAIN defaults to empty string, which may work for testing, but for set this right for production for cookies etc. to work.
    GANBARE_EMAIL_DOMAIN defaults to $GANBARE_SITE_DOMAIN
    GANBARE_SERVER_BINDING defaults to localhost:8080. When running inside a container, change this to 0.0.0.0:8080 for the site to be accessible from host.
    GANBARE_JQUERY defaults to /static/js/jquery.min.js. For production, try https://ajax.googleapis.com/ajax/libs/jquery/3.1.1/jquery.min.js
    GANBARE_FONT_URL defaults to /static/fonts/default.css. For production, try https://fonts.googleapis.com/css?family=Source+Sans+Pro:300
    GANBARE_AUDIO_DIR defaults to audio You shouldn't need to change this, but it's possible.
    GANBARE_IMAGES_DIR defaults to images You shouldn't need to change this, but it's possible.

During build, you need the following: 

    GANBARE_BUILDTIME_PEPPER=some 32-byte random value encoded with Base64 (usually 44 ASCII characters) for peppering the password hashes.

After creating an `.env` file, start the server:

    $ cargo run

Navigate to localhost:8080 with your browser. For debug builds, directories `static`, `migrations` and `templates`, `audio` and `images` are used runtime.
For release builds, only `static`, `audio` and `images` are used, as `migrations` and `templates` are compiled statically inside the binary.


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
