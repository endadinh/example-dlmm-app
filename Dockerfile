# ---------- Stage 1: Build frontend ----------
FROM node:24-alpine AS web
WORKDIR /app/web

# Yarn on Node 24 Corepack
RUN corepack enable && corepack prepare yarn@stable --activate

# Cache deps
COPY web/package.json web/yarn.lock ./
RUN yarn install --frozen-lockfile

# Copy source & build
COPY web/ .
RUN yarn build

# ---------- Stage 2: Build backend ----------
FROM rust:1.85.0 AS builder
WORKDIR /app

# Cache deps
COPY Cargo.toml Cargo.lock ./
RUN mkdir src && echo "fn main(){}" > src/main.rs
RUN cargo build --release || true

# Copy source & build
COPY src ./src
# ⬅️ Copy built web assets from Stage 1
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