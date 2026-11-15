FROM rust:1.94-slim AS build
WORKDIR /app
COPY . .
RUN cargo build --release --bin server --bin seed

FROM debian:bookworm-slim
RUN useradd --system --uid 10001 app
COPY --from=build /app/target/release/server /app/target/release/seed /usr/local/bin/
USER app
EXPOSE 8080
CMD ["server"]
