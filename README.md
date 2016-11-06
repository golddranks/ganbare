# Ganbare
A web service that helps me to do what I do. (Protip: Something related to Japanese language and language learning. And science.)

## How to setup a dev database
docker run --name ganbare-postgres -d --restart=unless-stopped -e POSTGRES_USER=`whoami` -p 127.0.0.1:5432:5432 postgres
diesel setup

Then setup user accounts with the `user` command line tool found in the `bin`.
