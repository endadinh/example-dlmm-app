# 🦀 Example DLMM App ( Saros ) — Axum + Vite.

![Rust](https://img.shields.io/badge/Rust-1.85+-934E01?logo=rust)
![Node](https://img.shields.io/badge/Node-22+-43853D?logo=node.js&logoColor=white)
![License](https://img.shields.io/badge/license-MIT-blue.svg)
![Status](https://img.shields.io/badge/status-dev-green)

---

> ⚠️ **This project is currently under active development.**  
> APIs, structures, and behaviors may change frequently without backward compatibility.  
> Use at your own risk until the first stable release is tagged.

---

> Development environment for [**Saros DLMM SDK**](https://github.com/saros-xyz/saros-dlmm-sdk-rs), combining Axum and Vite web frontend.  
> Managed with [`just`](https://github.com/casey/just) for clean and reproducible workflows.

---

## ⚙️ Prerequisites

Before running, make sure you have the following installed:

- 🦀 **Rust toolchain** (1.87.0)
  ```bash
  rustup override set 1.87.0
  ```
- 🧰 cargo-watch (for live reload)
  ```bash
  cargo install cargo-watch
  ```
- 💻 Node.js + Yarn
  ```bash
  cargo install cargo-watch
  ```
- 💻 Node.js + Yarn
  ```bash
  corepack enable
  yarn install
  ```
- ⚡ just — task runner for consistent commands
  ```bash
  cargo install just
  ```

## 🚀 Development

#### Run Full Dev Environment

Run both backend (Axum) and frontend (Vite) concurrently with live reload:

```bash
just dev
```

This will:

- Watch and rebuild the web app using Vite.
- Watch and restart the Rust backend using cargo-watch.
- Serve built web assets through the Axum server.

#### Deployment local :

```bash
cargo build --release
docker build -t dlmm-app .
docker run -p 8080:8080 dlmm-app
```
