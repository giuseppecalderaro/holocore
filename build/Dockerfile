FROM rust:1.64.0

WORKDIR /src
COPY . .
RUN rustup default nightly
RUN cargo build --release

RUN mkdir -p /exec/bin
RUN mkdir -p /exec/configs
RUN cp /src/target/release/main /exec/bin
RUN cp /src/static_data/prod/trades.toml /exec/configs
RUN cp /src/static_data/prod/klines.toml /exec/configs
RUN cp /src/static_data/prod/books.toml /exec/configs
RUN rm -rf /src
WORKDIR /exec
