FROM rust:alpine AS build
WORKDIR /build
RUN apk add --no-cache musl-dev perl make
COPY Cargo.toml Cargo.lock ./
COPY src ./src
RUN cargo build --release

FROM alpine:latest
RUN apk add --no-cache coreutils
COPY --from=build /build/target/release/ssh-storage-monitor /usr/local/bin/ssh-storage-monitor
EXPOSE 9091
ENTRYPOINT [ "/usr/local/bin/ssh-storage-monitor" ]