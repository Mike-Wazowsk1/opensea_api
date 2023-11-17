# Backend for Bitgesell
экенд сервер для Bitgesell Farm

## Перед запуском
- Необходимо,чтобы была установлена база данных postgresql и у нее были открыты порта для удаленного подключения
- Необходимо,чтобы был установлен docker

## Общая инструкция со старта
- [Установить postgresql](https://www.postgresql.org/download/linux/ubuntu/) и [настроить ее](#настройка-postgres)
- [Установить docker](https://docs.docker.com/engine/install/ubuntu/)
- [Запустить с помощью DockerHub сервис](#инструкция-докер)
## Настройка Postgres 
- Установить с помощью [инструкции](https://www.postgresql.org/download/linux/ubuntu/)
- Подключиться к бд
    ```bash
    sudo -u postgres psql
    ```
- Создать администратора в бд
    ```sql
    CREATE ROLE @user SUPERUSER LOGIN PASSWORD @password;
    CREATE DATABASE @user; 
    ```
- Настройка базы данных
    ```sql
    CREATE DATABASE opensea_api;
    ```
    - Подключиться к базе данных `opensea_api` 
        ```bash
        \c opensea_api
        ```
    - Создать таблицу `tokens`
        ```sql 
        CREATE TABLE tokens (index INTEGER PRIMARY KEY, id TEXT, count INTEGER, bracket INTEGER, level TEXT);
        ```
    - Создать таблицу `info`
        ```sql 
        CREATE TABLE info(hash TEXT PRIMARY KEY, wbgl INTEGER);
        ```
## Инструкция (без докера)
- Установить зависимости для Rust
  ```bash
  apt-get update && apt-get install build-essential curl libssl-dev libpq-dev pkg-config -y
  ```
- Установить Rust
    ```bash
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y && PATH="/root/.cargo/bin:${PATH}"
    ```
- Скомпилировать приложение
    ```bash
    cargo build --release
    ```
- Запустить приложение
  ```bash
  ./target/release/opensea_api
  ```

## Инструкция (Докер)
### Из исходников
- Скомпилировать образ 
    ```bash
    docker build . -t $CONTAINER_NAME
    ```
- Создать файл .env с данными для подключения к бд
    ```bash
    echo DATABASE_URL=postgres://$LOGIN:$PASSWORD@$IP:$PORT/opensea_api>.env
    ```
- Включить докер в автозагрузку 
    ```bash
    systemctl enable docker
    ```
- Запустить контейнер
  ```bash 
  docker run --env-file .env -p port:8080 -d --restart always $CONTAINER_NAME
  ```
### С помощью DockerHub
  - Запустить контейнер
  ```bash
  docker run --env-file .env -p port:8080 -d --restart always d4rkside445/opensea_api
  ```


- Чтобы запустить автоматическое заполнение таблицы `tokens` введите в терминале
 ```bash
curl http://127.0.0.1:8080/init_db
```


