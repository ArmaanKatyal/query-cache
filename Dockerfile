FROM rust:1.74.1

COPY ./ ./

RUN cargo build --release

EXPOSE 3000

CMD ["RUST_LOG=info ./target/release/query_cache"]
