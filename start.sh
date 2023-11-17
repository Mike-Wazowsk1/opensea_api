apt-get update -y
apt-get install -y libpq-dev
apt-get install -y libavfilter-dev
apt-get install -y libssl-dev
apt-get install -y pkg-config
apt-get install -y build-essential
cargo build --release
./make_service.sh
