FROM rust:bookworm AS builder
WORKDIR /usr/src/strong-sync-service
COPY . .
RUN cd strong-sync-service && RUSTFLAGS="-C debuginfo=2" cargo install --path . --debug

FROM debian:bookworm-slim
ARG VERSION=0.2.1
LABEL version="${VERSION}"
LABEL org.opencontainers.image.version="${VERSION}"
RUN apt-get update && apt-get install -y \
    openssl \
    ca-certificates \
    curl \
    cron

WORKDIR /usr/strong-sync-service
COPY --from=builder /usr/local/cargo/bin/strong-sync-service /usr/bin/strong-sync-service
COPY .env /.env
# will run the cron job every day at 18:00, 18:30, 19:00, 19:30, 20:00, and 20:30
RUN echo "0,30 18-20 * * * root RUST_BACKTRACE=1 RUST_LOG=debug /usr/bin/strong-sync-service >> /var/log/cron.log 2>&1" > /etc/cron.d/strong-sync-service

# Ensure the cron job file has proper permissions
RUN chmod 0644 /etc/cron.d/strong-sync-service && \
    chmod +x /usr/bin/strong-sync-service

# Create the log file so that it exists when cron writes to it
RUN touch /var/log/cron.log

COPY docker-entrypoint.sh /docker-entrypoint.sh
RUN chmod +x /docker-entrypoint.sh

CMD ["/docker-entrypoint.sh"]