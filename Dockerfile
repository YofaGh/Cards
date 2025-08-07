FROM rust:1-slim-bullseye AS builder

RUN sed -i 's/deb.debian.org/cdn-fastly.deb.debian.org/g' /etc/apt/sources.list
RUN apt-get update && apt-get install -y \
    build-essential \
    libpq-dev \
    pkg-config \
    openssl \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*
ENV PATH="/root/.cargo/bin:${PATH}"

RUN cargo install sqlx-cli

WORKDIR /usr/src/app

COPY Cargo.toml Cargo.lock ./
RUN mkdir src && echo "fn main() {}" > src/main.rs
RUN cargo build --release
RUN rm -rf src

COPY src ./src
COPY migrations ./migrations
RUN cargo build --release

FROM debian:bullseye-slim AS runner

WORKDIR /app

RUN apt-get update && apt-get install -y \
    libpq5 \
    openssl \
    && rm -rf /var/lib/apt/lists/*

RUN groupadd -r appgroup && useradd -r -g appgroup -s /bin/bash appuser

COPY --from=builder /usr/src/app/target/release/Cards .
COPY --from=builder /usr/local/cargo/bin/sqlx /usr/local/bin/
COPY --from=builder /usr/src/app/migrations ./migrations

COPY entrypoint.sh .
RUN chmod +x entrypoint.sh

COPY .env.example .env

RUN chown -R appuser:appgroup /app

USER appuser

EXPOSE 8000
EXPOSE 8080

ENTRYPOINT ["./entrypoint.sh"]

CMD ["./Cards"]