FROM rust:1.68 as planner

WORKDIR /app

RUN cargo install cargo-chef 
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

FROM rust:1.68 as cacher
WORKDIR /app
RUN cargo install cargo-chef
COPY --from=planner /app/recipe.json recipe.json
RUN cargo chef cook --release --recipe-path recipe.json

FROM rust:1.68 as builder
WORKDIR /app
COPY . .

COPY --from=cacher /app/target target
RUN cargo build --release --bin roadrunner

FROM rust:1.68

RUN mkdir app
# COPY --from=planner /run/secrets/cert.pem ./app/run/secrets/cert.pem
# COPY --from=planner /run/secrets/key.pem ./app/run/secrets/key.pem

COPY --from=builder /app/target/release/roadrunner ./

# ports and volumes
EXPOSE 8443/udp
EXPOSE 80
EXPOSE 443

# WORKDIR /app

CMD ["./roadrunner"]