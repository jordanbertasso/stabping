FROM rust:1.49.0-buster

# Install node and npm
RUN curl -sL https://deb.nodesource.com/setup_15.x | bash -
RUN apt-get install -y nodejs

# Install stabping
WORKDIR /stabping

COPY ./ ./

RUN cargo update

RUN cargo build --release

EXPOSE 5000
EXPOSE 5001

CMD ["/stabping/target/release/stabping"]

