FROM rust:1.64 AS builder
RUN mkdir src && touch src/lib.rs
COPY Cargo.* .
RUN cargo build --release
COPY src src
RUN touch /src/main.rs
RUN cargo build --release

FROM ubuntu
RUN apt-get update && apt-get install -y poppler-utils && rm -rf /var/lib/apt/lists/*
COPY --from=builder /target/release/pdf-bailiff /pdf-bailiff
USER nobody
ENTRYPOINT ["/pdf-bailiff"]
