#Я сдаюсь))))
FROM ubuntu
ENV TZ=Europe/Minsk
RUN ln -snf /usr/share/zoneinfo/$TZ /etc/localtime && echo $TZ > /etc/timezone

RUN apt-get update && apt-get install -y \
    build-essential \
    curl
RUN apt install build-essential
RUN apt install libssl-dev -y \
    libpq-dev -y    
RUN apt install pkg-config -y
RUN apt install postgresql postgresql-contrib -y
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y

ENV PATH="/root/.cargo/bin:${PATH}"

COPY . /opensea_api


WORKDIR /opensea_api


RUN cargo build --release

#RUN cargo install diesel_cli --no-default-features --features postgres
#RUN touch .env
#RUN echo DATABASE_URL=postgres://postgres:admin@127.0.0.1:5432/opensea_api > .env

#RUN diesel setup
#RUN diesel migration run
# RUN apt-get install nginx
# RUN service nginx start
# RUN iptables -I INPUT -p tcp -m tcp --dport 80 -j ACCEPT
# RUN  iptables -I INPUT -p tcp -m tcp --dport 443 -j ACCEPT
# RUN sudo chmod 700 /etc/ssl/private



CMD ["./target/release/opensea_api"]
# RUN curl 127.0.0.1:8000/init_db
