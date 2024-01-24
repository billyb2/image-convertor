FROM rust:1-slim-bookworm as builder

WORKDIR /
ADD . .
RUN apt-get update && apt-get install nasm -y
RUN cargo build --release

FROM debian:bookworm-slim

WORKDIR /app
COPY --from=builder /target/release/image_conversion .
CMD /app/image_conversion
