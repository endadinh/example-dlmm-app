# Justfile —  Saros DLMM SDK (Axum serves Web)
set shell := ["bash", "-cu"]
set dotenv-load := true

# ==== Directories ====
backend_dir := "."
web_dir := "./web"

# ==== Commands ====

default:
    @echo "🧭 Available tasks:"
    @just --summary

# 🚀 Run backend serving the web build (default mode)
dev:
    @echo "🚀 Starting DLMM full dev environment (Axum + Vite build)..."
    @echo "💻 Watching web + 🦀 backend concurrently..."
    cd {{web_dir}} && yarn install --silent
    npx concurrently \
        --kill-others-on-fail \
        --names "💻 WEB,🦀 API" \
        --prefix-colors "cyan.bold,yellow.bold" \
        "cd {{web_dir}} && yarn build --watch" \
        "cd {{backend_dir}} && cargo watch -x 'run -- start --web'"

# 💻 Frontend only (useful for UI tweaks)
web:
    @echo "💻 Building web in watch mode..."
    cd {{web_dir}} && yarn build --watch

# 🦀 Backend only
backend:
    @echo "🦀 Running backend (serves built web)..."
    cd {{backend_dir}} && cargo run -- start

# 🧱 Full build (for deployment)
build:
    @echo "🔨 Building frontend + backend for production..."
    cd {{web_dir}} && yarn build
    cd {{backend_dir}} && cargo build --release
    @echo "✅ Build complete: static files → web/dist, binary → target/release"

# 🧹 Clean workspace
clean:
    @echo "🧹 Cleaning artifacts..."
    cd {{web_dir}} && rm -rf dist node_modules
    cd {{backend_dir}} && cargo clean

# ✨ Format all code
fmt:
    @echo "🧼 Formatting Rust + Web..."
    cd {{backend_dir}} && cargo fmt
    cd {{web_dir}} && yarn prettier --write 'src/**/*.{ts,tsx,js,jsx,css,json}'