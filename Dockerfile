# Build stage
FROM rust:latest as builder
WORKDIR /usr/src/app
COPY . .
RUN cargo build --release
#RUN cargo test

# Runtime stage
FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y libssl3

RUN useradd -m chapterverse
WORKDIR /home/chapterverse

COPY --from=builder /usr/src/app/target/release/chapterverse .
COPY .env .
COPY --chown=chapterverse:chapterverse bibles/* ./bibles/
COPY --chown=chapterverse:chapterverse channels/* ./channels/

RUN chown chapterverse:chapterverse chapterverse .env

USER chapterverse
CMD ["./chapterverse"]

