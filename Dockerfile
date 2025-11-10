# ---------- Stage 1: Build frontend ----------
FROM node:24-alpine AS web
WORKDIR /app/web

# Enable corepack (for yarn)
RUN corepack enable

# Copy deps and install
COPY web/package.json web/yarn.lock ./
RUN yarn install --immutable

# Copy source & build
COPY web/ .
RUN yarn build

# ---------- Stage 2: Build backend ----------
FROM rust:1.85.0 AS builder
WORKDIR /app

# Cache Cargo deps & prebuild
COPY Cargo.toml Cargo.lock ./
RUN cargo fetch

# Copy source + built web
COPY src ./src
COPY --from=web /app/web/dist ./web/dist

RUN cargo build --release

# ---------- Stage 3: Runtime ----------
FROM debian:bookworm-slim
WORKDIR /app

RUN apt-get update && apt-get install -y --no-install-recommends ca-certificates && rm -rf /var/lib/apt/lists/*

COPY --from=builder /app/target/release/dlmm-app-interface /app/dlmm-app-interface
COPY --from=web /app/web/dist /app/web/dist

EXPOSE 8080
CMD ["./dlmm-app-interface", "start", "--web"]