# ---------- Build frontend ----------
FROM node:24-alpine AS web
WORKDIR /app/web
COPY web/package*.json ./
RUN yarn install --frozen-lockfile
COPY web/ .
RUN yarn build

# ---------- Build backend ----------
FROM rust:1.85.0 AS builder
WORKDIR /app
# copy Cargo files separately to cache dependencies
COPY Cargo.toml Cargo.lock ./
COPY src ./src
COPY web/dist ./web/dist
RUN cargo build --release

# ---------- Runtime ----------
FROM debian:bookworm-slim
WORKDIR /app
# copy binary
COPY --from=builder /app/target/release/dlmm-app-interface /app/dlmm-app-interface
# copy web files (if backend serves them)
COPY --from=web /app/web/dist /app/web/dist
# expose port
EXPOSE 8080
CMD ["./dlmm-app-interface", "start", "--web"]