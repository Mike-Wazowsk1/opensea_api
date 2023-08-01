# Backend for Bitgesell
This repo contains source code for backend of Bitgesell. 

## Prerequisites
- Installed Postgres Psql with initiallized db's (inital scripts can be found /migrations/00000000000000_diesel_initial_setup/up.sql)
- Intalled docker on hosted server

## Instruction (No docker)
- Install prerequisites for Rust
  ```bash
  apt-get update && apt-get install build-essential curl libssl-dev libpq-dev pkg-config -y
  ```
- Install Rust
    ```bash
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y && PATH="/root/.cargo/bin:${PATH}"
    ```
- Build apllciation
    ```bash
    cargo build --release
    ```
- Run apllciation
  ```bash
  ./target/release/opensea_api
  ```

## Instruction (Docker)
### From source
- Build 
    ```bash
    docker build . -t $CONTAINER_NAME
    ```
- Create .env file with link to db
    ```bash
    echo DATABASE_URL=postgres://$LOGIN:$PASSWORD@$IP:$PORT/opensea_api>.env
    ```
- Enable docker
    ```bash
    systemctl enable docker
    ```
- Run container 
  ```bash 
  docker run --env-file .env -p port:8080 -d --restart always $CONTAINER_NAME
  ```
### From DockerHub
  - Run container
  ```bash
  docker run --env-file .env -p port:8080 -d --restart always d4rkside445/opensea_api
  ```

## Postgres setup
- Install postgres from [instruction](https://www.postgresql.org/download/linux/ubuntu/)
- Setup postgres user 
    ```sql
    CREATE ROLE @user SUPERUSER LOGIN PASSWORD @password;
    CREATE DATABASE @user; 
    ```
- Setup postgres Db's
    ```sql
    CREATE DATABASE opensea_api;
    ```
    - Connect to `opensea_api` db
        ```bash
        \c opensea_api
        ```
    - Create `tokens` db
        ```sql 
        CREATE TABLE tokens (index INTEGER PRIMARY KEY, id TEXT, count INTEGER, bracket INTEGER, level TEXT);
        ```
    - Create `info` db
        ```sql 
        CREATE TABLE info(hash TEXT PRIMARY KEY, wbgl INTEGER);
        ```
- To init table `tokens` use this command on backend server
  ```bash
  curl http://127.0.0.1:8080/init_db
  ```


