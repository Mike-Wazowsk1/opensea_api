# Используем базовый образ Ubuntu
FROM ubuntu

# Установка зависимостей
RUN apt-get update && apt-get install -y \
    build-essential \
    curl \
    pkg-config

# Установка Rustup
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y

# Добавлениеной окружения PATH для доступа к инструментам Rust
ENV PATH="/root/.cargo/bin:${PATH}"

# Копирование исходного кода сервера в контейнер
COPY . /opensea_api

# Переход в директорию с исходным кодом
WORKDIR /opensea_api
RUN apt install libssl-dev -y \
libpq-dev -y

# Сборка сервера
RUN cargo build --release

# Определение команды запуска сервера при старте контейнера
CMD ["./target/release/opensea_api"]
