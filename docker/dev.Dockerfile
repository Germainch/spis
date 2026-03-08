# Stage 1: Compile
FROM rust:latest AS builder
WORKDIR /app

COPY Cargo.toml Cargo.lock rust-toolchain.toml ./
COPY build.rs ./
COPY src/ src/
COPY migrations/ migrations/
COPY templates/ templates/

RUN cargo build --release --locked
RUN cp target/release/spis /usr/bin/spis

# Stage 2: Runtime (mirrors existing Dockerfile)
FROM nginx:1.29.5

RUN apt-get update && apt-get install -y \
    supervisor \
    ffmpeg \
    && rm -rf /var/lib/apt/lists/*

ENV \
    NGINX_PORT=8080 \
    RUST_LOG=error,spis=info \
    SPIS_MEDIA_DIR=/var/lib/spis/media \
    SPIS_DATA_DIR=/var/lib/spis/data \
    SPIS_API_MEDIA_PATH=/assets/media \
    SPIS_API_THUMBNAIL_PATH=/assets/thumbnails
ENV \
    SPIS_SERVER_SOCKET=${SPIS_DATA_DIR}/spis.sock

COPY docker/docker-entrypoint.d/* /docker-entrypoint.d/
COPY docker/supervisor/* /etc/supervisor/

COPY --from=builder /usr/bin/spis /usr/bin/spis
RUN chmod +x /usr/bin/spis && \
    /usr/bin/spis --version && \
    mkdir -p ${SPIS_MEDIA_DIR} ${SPIS_DATA_DIR}

CMD ["supervisord", "-c", "/etc/supervisor/supervisord.conf"]
