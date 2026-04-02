<div align="center">

# IPA Tool

**A modern IPA file download and management tool**

[![License: MIT](https://img.shields.io/badge/License-MIT-green.svg)](https://opensource.org/licenses/MIT)
[![Docker](https://img.shields.io/badge/Docker-Ready-blue.svg)](https://www.docker.com/)

</div>

## ✨ Features

- 🔍 **App Search** — Search by app name, Bundle ID, or App ID
- 📦 **Version Management** — Browse and download historical app versions
- 🛒 **Account Management** — Manage multiple Apple IDs with per-account App Store region selection; credentials encrypted with AES-256-GCM
- 📥 **Downloads** — Direct download with progress tracking and a download queue
- 📲 **OTA Installation** — Install IPAs over-the-air (requires HTTPS)
- 🔐 **Secure Storage** — Local SQLite database; encrypted credential storage
- 🎨 **Modern UI** — Vue 3 + Element Plus, responsive design, dark mode
- ⚡ **High-performance backend** — Rust + Actix-web, async, memory-safe
- 🖥️ **Desktop App** — Optional Tauri wrapper for a native desktop experience (single window, no browser required)

## 🚀 Quick Start

### 🐳 Docker (Recommended)

```bash
# Clone
git clone https://github.com/iamjairo/ipaTool.git
cd ipaTool

# Start (detached)
docker-compose up -d

# View logs
docker-compose logs -f

# Open in browser
open http://localhost:8080

# Stop
docker-compose down
```

### 💻 Local Development (single terminal)

**Prerequisites:** Node 18+, pnpm 9+, Rust 1.70+

```bash
# Install frontend dependencies
pnpm install

# Run the Rust backend + Vite frontend together (one terminal!)
pnpm start
```

This starts:
- **Rust backend** on `http://localhost:8080`
- **Vite dev server** on `http://localhost:3000` (proxies `/api` to the backend)

Open **http://localhost:3000** in your browser.

#### Run servers separately (two terminals)

```bash
# Terminal 1 – Rust backend
pnpm dev:rust

# Terminal 2 – Vue frontend
pnpm dev
```

### 🖥️ Desktop App (Tauri)

Run as a native desktop window — no browser needed.

**Additional prerequisites:** [Tauri system dependencies](https://tauri.app/start/prerequisites/)

```bash
# Build the Rust server first
pnpm build:rust

# Launch the desktop app in dev mode
pnpm tauri:dev

# Build a distributable desktop installer
pnpm tauri:build
```

The Tauri app automatically starts the backend server on launch and shuts it down on exit.

### 🏗️ Production Build (manual)

```bash
# 1. Build the Vue frontend
pnpm build

# 2. Build the Rust backend
pnpm build:rust

# 3. Run the server (serves both the UI and the API)
./server/target/release/server
```

Open **http://localhost:8080**.

## 🔐 Default Credentials

| Field | Value |
|-------|-------|
| Username | `admin` |
| Password | `admin` |

> ⚠️ You will be prompted to change these on first login.

## 🌍 Apple ID Region Support

When adding an Apple ID account you can now select its **App Store region** from a comprehensive dropdown (80+ countries). The selected region is stored alongside the account and used for all searches, version lookups, and downloads — so a Netherlands Apple ID will correctly search the Dutch App Store.

If Apple returns region information during authentication it takes priority; otherwise the manually selected region is used.

## 🔌 API Reference

All API endpoints are under `/api` and require an active admin session (cookie-based).

| Method | Path | Description |
|--------|------|-------------|
| POST | `/api/auth/login` | Admin login |
| POST | `/api/auth/logout` | Admin logout |
| GET | `/api/auth/me` | Current user |
| POST | `/api/auth/change-password` | Change admin password |
| GET | `/api/accounts` | List Apple ID accounts |
| POST | `/api/login` | Add Apple ID account |
| DELETE | `/api/accounts/:token` | Remove Apple ID account |
| GET | `/api/versions` | Query app version history |
| GET | `/api/search` | Search App Store |
| GET | `/api/app-meta` | App metadata lookup |
| POST | `/api/download` | Start a download |
| GET | `/api/jobs` | List download jobs |
| GET | `/api/ipa-files` | List stored IPA files |
| DELETE | `/api/ipa-files/:id` | Delete a stored IPA |
| GET | `/api/health` | Health check |

## 🛠️ Tech Stack

| Layer | Technology |
|-------|-----------|
| Frontend | Vue 3, Vite, Element Plus, TailwindCSS, Pinia |
| Desktop | Tauri 2 (optional) |
| Backend | Rust, Actix-web, SQLite, OpenSSL |
| Deployment | Docker, GitHub Actions |

## 📜 License

[MIT](LICENSE)
