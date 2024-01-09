FROM rustlang/rust:nightly-buster AS planner
WORKDIR /app
RUN cargo install cargo-chef
COPY . .
RUN cargo chef prepare --recipe-path recipe.json && cat recipe.json


FROM rustlang/rust:nightly-buster as cacher
RUN cargo install cargo-chef

WORKDIR /app
COPY --from=planner /app/recipe.json recipe.json
RUN cargo chef cook --release --recipe-path recipe.json


FROM cacher as builder

WORKDIR /app
COPY . .
COPY --from=cacher /app/target target

RUN cargo build --release


FROM gcr.io/distroless/cc-debian12:latest as runner

COPY --from=builder /app/target/release/rathole-operator /app/
WORKDIR /app

# Set any required env variables and
ENV RUST_LOG="info"
EXPOSE 8080
# Run the server
CMD ["/app/rathole-operator"]