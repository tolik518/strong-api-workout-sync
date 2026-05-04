# Dockerfile
#
# Build and run without Docker Compose. You need a ClickHouse server running and accessible from the container.
# You should use the docker compose from https://github.com/tolik518/strong-api-workout-sync additionally to
# have a ClickHouse server running with the workout data.
#
#   docker build -t heatmap .
#   docker run -d --name heatmap \
#     --restart unless-stopped \
#     --network service-network \
#     -p 1337:80 \
#     -e CH_HOST=clickhouse-server \
#     -e CH_USER=tolik518 \
#     -e CH_PASS=admin \
#     -e CH_DB=workouts \
#     -e DEBUG=1 \
#     -v "$PWD/workout-heatmap.php":/var/www/html/index.php:ro \
#     heatmap
#
#   Open: http://localhost:1337/
#
FROM php:8.2-apache

# GD-Extension für PNG/Text
RUN apt-get update \
 && apt-get install -y libpng-dev libjpeg-dev libfreetype6-dev \
 && docker-php-ext-configure gd --with-freetype --with-jpeg \
 && docker-php-ext-install -j$(nproc) gd \
 && rm -rf /var/lib/apt/lists/*

COPY workout-heatmap.php /var/www/html/workout-heatmap.php

RUN echo "ServerName localhost" >> /etc/apache2/apache2.conf \
 && echo "DirectoryIndex workout-heatmap.php" >> /etc/apache2/conf-enabled/docker-php.conf