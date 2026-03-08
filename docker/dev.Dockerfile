# Stage 1: Compile
FROM rust:latest AS builder
WORKDIR /app

RUN apt-get update && apt-get install -y sqlite3 && rm -rf /var/lib/apt/lists/*

COPY Cargo.toml Cargo.lock rust-toolchain.toml ./
COPY build.rs ./
COPY migrations/ migrations/
COPY src/ src/
COPY templates/ templates/

# Create a temporary DB with schema applied so sqlx can check queries at compile time
RUN for f in $(ls migrations/*.sql | sort); do sqlite3 /tmp/spis-build.db < "$f"; done
ENV DATABASE_URL=sqlite:///tmp/spis-build.db

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
