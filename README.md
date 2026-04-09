# ⚡ Blitz-Load

**Blitz-Load** is a high-performance load-testing platform built in **Rust**, featuring both a **CLI tool** with an embedded Python 3.14 interpreter and a **full-stack web dashboard** with real-time telemetry.

---

## 🚀 Core Philosophy

Traditional load-testing tools usually fall into one of two extremes:

- **Fast but static** — sending the same payload repeatedly
- **Flexible but slow** — interpreted runtimes struggling with high concurrency

**Blitz-Load** delivers the best of both worlds:

- 🦀 **Rust Muscle**
  Handles thousands of concurrent TCP connections using non-blocking I/O via Tokio.

- 🐍 **Python Brain** *(CLI mode)*
  Enables dynamic, stateful test logic (unique users, CSV-fed data, conditional flows) without recompiling the engine.

- 🌐 **Web Dashboard**
  Configure, launch, and monitor load tests from an interactive browser UI with live-streaming metrics.

---

## 🏗️ Architecture

Blitz-Load ships as **two binaries** from a single Cargo workspace:

| Binary | Entry Point | Description |
|---|---|---|
| `blitz` | `src/main.rs` | Standalone CLI with embedded PyO3 |
| `blitz-server` | `src/bin/server.rs` | Axum HTTP server + React frontend |

Both binaries share the core load-testing engine defined in `src/engine.rs`.

```
┌──────────────────────────────────────────────────────────────┐
│                      Blitz-Load Engine                       │
│          (engine.rs — Tokio + reqwest + Semaphore)           │
│                                                              │
│  • Spawns one async task per request                         │
│  • Semaphore-controlled concurrency (max 50)                 │
│  • Lock-free metrics (AtomicUsize / AtomicU64)               │
│  • Broadcast-channel progress events (500ms ticks)           │
│  • Latency percentiles: p50, p95, p99                        │
│  • Thundering-herd spread measurement (µs)                   │
└────────────────┬─────────────────────────┬───────────────────┘
                 │                         │
        ┌────────▼────────┐       ┌────────▼─────────┐
        │   CLI (blitz)   │       │  Server (axum)   │
        │                 │       │                   │
        │ • Clap args     │       │ POST /api/run     │
        │ • PyO3 payloads │       │ GET  /api/stream/ │
        │ • Progress bar  │       │ GET  /api/health  │
        │ • Terminal stats│       │ Serves frontend/  │
        └─────────────────┘       └────────┬──────────┘
                                           │
                                  ┌────────▼──────────┐
                                  │  React Dashboard  │
                                  │  (Vite + TW v4)   │
                                  │                   │
                                  │ • ConfigPanel     │
                                  │ • LiveDashboard   │
                                  │   (SSE + Recharts)│
                                  │ • ResultsSummary  │
                                  └───────────────────┘
```

---

## 🛠️ Technical Deep Dive

### Engine (`src/engine.rs`)

- **Async Runtime** — Built on **Tokio**, leveraging an M:N threading model to simulate hundreds of virtual users efficiently.
- **Concurrency Control** — `Semaphore`-gated; up to 50 concurrent in-flight requests, capped at 500 total per test.
- **Memory Model** — `Arc` for shared ownership, `AtomicUsize` for lock-free counters, `Mutex` only for the latency vec.
- **Payload Scenarios** — Three built-in generators:
  - `UserRegistration` — UUID-based unique user payloads
  - `SimpleGet` — No-body GET requests
  - `CustomJson` — User-supplied JSON body
- **Progress Broadcasting** — `tokio::sync::broadcast` channel emits `Progress` events every 500ms and a final `Done` event with full statistics.

### CLI (`src/main.rs`)

- **PyO3 FFI** — Embeds CPython via PyO3 with ABI3 forward compatibility (developed against Python 3.14).
- **Dynamic Payloads** — Each request calls `get_payload()` from `scripts/scenario.py`, producing unique data per request.
- **UDP Trigger** — All workers prepare payloads then wait on a `Notify` barrier, released by a UDP signal on port 9000 for synchronized thundering-herd testing.
- **Terminal Output** — `indicatif` progress bar, p95/p99 latency stats, RPS, and start-time spread analysis.

### Web Server (`src/bin/server.rs`)

- **Framework** — Axum 0.7 with `tower-http` CORS and static-file serving.
- **State** — `DashMap<Uuid, TestEntry>` for concurrent test management; each test gets a unique UUID.
- **SSE Streaming** — `/api/stream/:id` returns a Server-Sent Events stream via `BroadcastStream`. Clients can reconnect freely without crashing the worker.
- **Auto-cleanup** — Finished tests are retained for 5 minutes (for late reconnects), then evicted.

### Frontend (`frontend/`)

- **Stack** — React 19 + Vite 8 + Tailwind CSS v4
- **Design** — Neo-brutalism aesthetic with Space Grotesk / JetBrains Mono typography
- **Components**:
  - `Hero` — Landing page with call-to-action
  - `ConfigPanel` — URL, method, requests, concurrency, and scenario configuration
  - `LiveDashboard` — Real-time SSE consumer with Recharts latency graph, throughput gauge, and terminal-style log
  - `ResultsSummary` — Final report: RPS, success rate, p50/p95/p99 latencies, and thundering-herd spread
- **Animations** — Framer Motion view transitions with `AnimatePresence`

---

## ✨ Key Features

| Feature | Description |
|---|---|
| **p50 / p95 / p99 Latency** | Reveals long-tail performance issues hidden by averages |
| **Dynamic Payload Generation** | Every request is unique — prevents cache-warmed benchmarks |
| **Real-Time Dashboard** | Live SSE-powered metrics with latency trend charts |
| **Thundering-Herd Spread** | Measures µs-level synchronization of request dispatch |
| **Dual Interface** | Full-featured CLI *and* web dashboard from one codebase |
| **Scenario Templates** | UUID user registration, simple GET, or custom JSON |
| **Safety Caps** | Max 500 requests / 50 concurrency enforced server-side |

---

## 📦 Prerequisites

- **Rust** — Edition 2021 (stable toolchain)
- **Node.js** — v18+ with npm (for the frontend)
- **Python 3.10+** — Development headers required *(CLI binary only)*

---

## ▶️ Quick Start

### Option 1: Web Dashboard (recommended)

```bash
# 1. Build the frontend
cd frontend
npm install
npm run build
cd ..

# 2. Build and run the server
cargo build --release --bin blitz-server
./target/release/blitz-server
```

Open **http://localhost:3000** in your browser to configure and run load tests.

> **Tip:** Set the `PORT` environment variable to change the server port.

#### Development Mode (hot-reload)

Run the Rust server and Vite dev server simultaneously:

```bash
# Terminal 1 — Backend
cargo run --bin blitz-server

# Terminal 2 — Frontend (proxies /api → localhost:3000)
cd frontend
npm run dev
```

The Vite dev server runs on **http://localhost:5173** and proxies API calls to the Rust backend.

### Option 2: CLI with Python Scripts

```bash
# Build
cargo build --release --bin blitz

# Run
./target/release/blitz --url https://your-api.com/endpoint --requests 200 --concurrency 20
```

Customize payload logic by editing `scripts/scenario.py`.

---

## 🔌 API Reference

The `blitz-server` exposes three endpoints under `/api`:

### `POST /api/run`

Start a new load test. Returns a `test_id` for streaming.

**Request body:**
```json
{
  "url": "https://httpbin.org/post",
  "method": "post",
  "requests": 100,
  "concurrency": 10,
  "scenario": "user_registration",
  "custom_body": null
}
```

| Field | Type | Default | Description |
|---|---|---|---|
| `url` | string | *(required)* | Target endpoint (must start with `http://` or `https://`) |
| `method` | string | `"post"` | HTTP method: `"get"` or `"post"` |
| `requests` | number | `100` | Total requests to send (max 500) |
| `concurrency` | number | `10` | Concurrent workers (max 50) |
| `scenario` | string | `"user_registration"` | `"user_registration"`, `"simple_get"`, or `"custom_json"` |
| `custom_body` | string? | `null` | JSON body for `custom_json` scenario |

**Response:**
```json
{ "test_id": "550e8400-e29b-41d4-a716-446655440000" }
```

### `GET /api/stream/:id`

Server-Sent Events stream for a running test. Emits two event types:

- **`progress`** — Every ~500ms with `completed`, `total`, `successes`, `failures`, `rps`, `avg_latency_ms`, `p95_latency_ms`
- **`done`** — Final event with full statistics including `p50`, `p95`, `p99`, `spread_us`, and `total_time_ms`

### `GET /api/health`

Returns `"ok"` — useful for readiness probes.

---

## 📁 Project Structure

```text
.
├── .cargo/
│   └── config.toml              # PyO3 ABI3 forward-compat flag
├── src/
│   ├── main.rs                  # CLI binary (Clap + PyO3 + Tokio)
│   ├── engine.rs                # Shared load-test engine
│   ├── lib.rs                   # Library crate (re-exports engine)
│   └── bin/
│       └── server.rs            # Axum web server binary
├── scripts/
│   ├── scenario.py              # Default Python payload generator
│   └── basic_test.py            # Simple test script example
├── frontend/
│   ├── index.html               # Entry HTML (Space Grotesk + JetBrains Mono)
│   ├── package.json             # React 19, Vite 8, Tailwind CSS v4, Recharts
│   ├── vite.config.js           # Dev proxy to Rust backend
│   ├── .env.development         # Dev environment variables
│   ├── .env.example             # Env template
│   └── src/
│       ├── main.jsx             # React entry point
│       ├── App.jsx              # View router (Hero → Config → Live → Results)
│       ├── App.css              # Neo-brutalism design tokens
│       ├── index.css            # Global styles + Tailwind imports
│       ├── ThemeContext.jsx      # Dark/light theme provider
│       ├── api/
│       │   └── index.js         # Fetch wrapper for /api/run
│       └── components/
│           ├── NavBar.jsx       # Top navigation bar
│           ├── Hero.jsx         # Landing page with CTA
│           ├── ConfigPanel.jsx  # Test configuration form
│           ├── LiveDashboard.jsx # Real-time SSE metrics + Recharts graph
│           └── ResultsSummary.jsx # Final test report
├── Cargo.toml                   # Workspace: blitz + blitz-server binaries
└── README.md
```

---

## 📊 Real-World Case Study
### Node.js + MongoDB Atlas

**Scenario:**
Stress-tested a local Node.js registration endpoint.

- **Concurrency:** 20 workers
- **Endpoint:** `/auth/register`

**Observations:**
- ~80% request failure rate
- p99 latency exceeded **1.9 seconds**

**Diagnosis:**
- **CPU Bottleneck:** Bcrypt password hashing saturation
- **I/O Bottleneck:** Cloud network latency (Local → MongoDB Atlas)

**Insight:**
Single-threaded Node.js event loops require worker threads and connection pooling for cryptographic workloads.

---

## 📄 License

This project is open source. See the repository for license details.