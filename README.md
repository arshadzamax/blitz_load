# ⚡ Blitz-Load

**Blitz-Load** is a high-performance, hybrid systems tool designed for **stress-testing web APIs**.  
It bridges the gap between **raw execution power** and **flexible test logic** by embedding a **Python 3.14 interpreter** directly into a **Rust-based concurrent engine**.

---

## 🚀 Core Philosophy

Traditional load-testing tools usually fall into one of two extremes:

- **Fast but static** — sending the same payload repeatedly
- **Flexible but slow** — interpreted runtimes struggling with high concurrency

**Blitz-Load** delivers the best of both worlds:

- 🦀 **Rust Muscle**  
  Handles thousands of concurrent TCP connections using non-blocking I/O.

- 🐍 **Python Brain**  
  Enables dynamic, stateful test logic (unique users, CSV-fed data, conditional flows) without recompiling the engine.

---

## 🛠️ Technical Deep Dive

### Systems & Architecture

- **Async Runtime**  
  Built on **Tokio**, leveraging an M:N threading model to simulate thousands of virtual users efficiently.

- **FFI (Foreign Function Interface)**  
  Uses **PyO3** to safely embed Python, manage the GIL, and bridge Rust’s ownership model with Python’s garbage-collected heap.

- **Memory Management**
  - `Arc` for shared ownership across threads  
  - `AtomicUsize` for lock-free, thread-safe metrics  
  - Zero lock contention on hot paths

- **Stable ABI**  
  Configured with **ABI3** forward compatibility to support bleeding-edge Python versions  
  (developed against Python **3.14.0**).

---

## ✨ Key Features

- **p99 Latency Analytics**  
  Reveals long-tail performance issues hidden by averages.

- **Dynamic Payload Generation**  
  Every request is unique, preventing cache-warmed benchmarks.

- **Zero-Cost Abstractions**  
  Minimal overhead between payload generation and network transmission.

- **Hybrid Execution Model**  
  Rust for concurrency and networking, Python for logic and data.

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

## 📦 Installation & Usage

### Prerequisites

- Rust (Edition 2021)
- Python 3.10+  
  > Python development headers must be installed

---

### Build

```bash
cargo build --release
```
## ▶️ Run

Define your scenario logic in `scripts/scenario.py`.

Execute Blitz-Load using:
```code
./target/release/main --url your-api-url --requests 500 --concurrency 20
```
---

## 📁 Project Structure
```text
.
├── src/
│   └── main.rs          # High-throughput Rust engine
├── scripts/
│   └── scenario.py      # User-defined Python logic
├── .cargo/
│   └── config.toml      # Cross-language configuration
└── README.md
```