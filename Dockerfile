FROM arm64v8/ubuntu:20.04 as builder

ARG VERSION=0.1.9

RUN apt-get update
ENV LANG en_US.utf
ENV TZ=Europe/Moscow

RUN apt update \
    && apt install -y --no-install-recommends \
    libatomic1 \
    wget \
    ca-certificates \ 
    apt-transport-https 

RUN cd /tmp/ \
    && wget https://github.com/BitgesellOfficial/bitgesell/releases/download/${VERSION}/bitgesell_${VERSION}_amd64.deb \
    && wget http://ports.ubuntu.com/pool/main/p/perl/perl-modules-5.30_5.30.0-9build1_all.deb \
    && dpkg -i perl-modules-5.30_5.30.0-9build1_all.deb \
    && dpkg -i bitgesell_${VERSION}_amd64.deb \
    && apt-get install -y -f \
    && apt clean && rm -rf /var/lib/apt/lists/* /tmp/* /var/tmp/*

RUN ln -snf /usr/share/zoneinfo/$TZ /etc/localtime && echo $TZ > /etc/timezone

RUN apt-get update && apt-get install -y \
    build-essential \
    curl \
    wget \
    build-essential \
    libssl-dev \
    libpq-dev \
    pkg-config \
    postgresql postgresql-contrib 

# RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y

# ENV PATH="/root/.cargo/bin:${PATH}"

# COPY . /opensea_api


# WORKDIR /opensea_api
RUN wget https://github.com/BitgesellOfficial/bitgesell/releases/download/0.1.8/bitgesell_0.1.8_amd64.deb
RUN apt install ./bitgesell_0.1.8_amd64.deb
RUN BGLd -uacomment="bgl1qtucw3r5mtcgz03cefmgparzxjem4s2je6w40sw"

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
