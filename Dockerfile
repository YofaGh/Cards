FROM debian:bookworm-slim

WORKDIR /app

COPY /target/release/Cards /app/Cards
COPY cert.pem /app/cert.pem
COPY key.pem /app/key.pem
COPY .env /app/.env

RUN chmod +x /app/Cards

EXPOSE 8080 8000

CMD ["./Cards"]