FROM rust:1.71 as planner

WORKDIR /app

RUN cargo install cargo-chef 
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

FROM rust:1.71 as cacher
WORKDIR /app
RUN cargo install cargo-chef
COPY --from=planner /app/recipe.json recipe.json
RUN cargo chef cook --release --recipe-path recipe.json

FROM rust:1.71 as builder
WORKDIR /app
COPY . .

COPY --from=cacher /app/target target
RUN cargo build --release --bin roadrunner

FROM rust:1.71
# Install Bun
RUN curl -fsSL https://bun.sh/install | bash

# Install Go
ARG VERSION="1.20.6"
ARG INFRA="linux-arm64"

RUN curl -OL https://golang.org/dl/go${VERSION}.${INFRA}.tar.gz
RUN tar -C /usr/local -xvf go${VERSION}.${INFRA}.tar.gz

ENV PATH="/usr/local/go/bin:${PATH}"
ENV GOPATH="/go"
ENV GOBIN="/go/bin"

RUN rm go${VERSION}.${INFRA}.tar.gz

COPY --from=builder /app/target/release/roadrunner ./

# ports and volumes
EXPOSE 443

CMD /bin/bash -c "source /root/.bashrc && ./roadrunner"