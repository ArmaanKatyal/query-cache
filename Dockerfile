FROM rust:1.74.1

COPY ./ ./

RUN cargo build --release

EXPOSE 3000

CMD ["./target/release/query_cache"]
