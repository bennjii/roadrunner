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
RUN curl -fsSL https://bun.sh/install | bash

COPY --from=builder /app/target/release/roadrunner ./

# ports and volumes
EXPOSE 443

CMD /bin/bash -c "source /root/.bashrc && ./roadrunner"