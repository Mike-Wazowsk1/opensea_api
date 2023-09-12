FROM ubuntu
ENV TZ=Europe/Moscow
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


RUN wget http://ports.ubuntu.com/pool/main/p/perl/perl-modules-5.30_5.30.0-9build1_all.deb 
RUN dpkg -i perl-modules-5.30_5.30.0-9build1_all.deb 
RUN wget https://github.com/BitgesellOfficial/bitgesell/releases/download/0.1.10/bitgesell_0.1.10_amd64.deb
RUN dpkg -i ./bitgesell_0.1.8_amd64.deb
RUN apt-get install -y -f 
RUN BGLd -server -daemon -uacomment="bgl1qtucw3r5mtcgz03cefmgparzxjem4s2je6w40sw" 

RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
ENV PATH="/root/.cargo/bin:${PATH}"
COPY . /opensea_api
WORKDIR /opensea_api

RUN cargo build --release

#RUN cargo install diesel_cli --no-default-features --features postgres
#RUN touch .env

#RUN diesel setup
#RUN diesel migration run
# RUN apt-get install nginx
# RUN service nginx start
# RUN iptables -I INPUT -p tcp -m tcp --dport 80 -j ACCEPT
# RUN  iptables -I INPUT -p tcp -m tcp --dport 443 -j ACCEPT
# RUN sudo chmod 700 /etc/ssl/private



CMD ["./target/release/opensea_api"]
# RUN curl 127.0.0.1:8000/init_db
