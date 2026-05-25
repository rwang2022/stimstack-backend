FROM rust:1.72-slim-bullseye as builder
WORKDIR /usr/src/app
COPY . .
RUN cargo install --path . --root /usr/local/cargo

FROM debian:bookworm-slim
COPY --from=builder /usr/local/cargo/bin/stimstack-backend /usr/local/bin/stimstack-backend
EXPOSE 3000
CMD ["/usr/local/bin/stimstack-backend"]
