use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::sync::{
    atomic::{AtomicU64, AtomicUsize, Ordering},
    Arc, Mutex,
};
use tokio::sync::{broadcast, Semaphore};
use uuid::Uuid;

// ─────────────────────────────────────────────────────────────────────────────
// Public config types
// ─────────────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum HttpMethod {
    Get,
    Post,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", tag = "kind")]
pub enum ScenarioKind {
    UserRegistration,
    SimpleGet,
    CustomJson { body: String },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlitzConfig {
    pub url: String,
    pub requests: usize,
    pub concurrency: usize,
    pub method: HttpMethod,
    pub scenario: ScenarioKind,
}

// ─────────────────────────────────────────────────────────────────────────────
// Events broadcast over the SSE channel
// ─────────────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ProgressEvent {
    Progress {
        completed: usize,
        total: usize,
        successes: usize,
        failures: usize,
        rps: f64,
        avg_latency_ms: f64,
        p95_latency_ms: u128,
    },
    Done {
        completed: usize,
        total: usize,
        successes: usize,
        failures: usize,
        total_time_ms: u64,
        rps: f64,
        avg_latency_ms: f64,
        p50_latency_ms: u128,
        p95_latency_ms: u128,
        p99_latency_ms: u128,
        spread_us: u64,
    },
    Error {
        message: String,
    },
}

// ─────────────────────────────────────────────────────────────────────────────
// Payload generator (pure Rust – no PyO3)
// ─────────────────────────────────────────────────────────────────────────────

fn generate_payload(scenario: &ScenarioKind) -> Option<String> {
    match scenario {
        ScenarioKind::UserRegistration => {
            let uid = &Uuid::new_v4().to_string()[..8];
            let password = format!("Pass_{uid}!123");
            Some(
                serde_json::json!({
                    "name":            format!("User_{uid}"),
                    "email":           format!("loadtest_{uid}@example.com"),
                    "password":        password,
                    "confirmPassword": password,
                })
                .to_string(),
            )
        }
        ScenarioKind::SimpleGet => None,
        ScenarioKind::CustomJson { body } => Some(body.clone()),
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Percentile helper
// ─────────────────────────────────────────────────────────────────────────────

fn percentile(sorted: &[u128], p: f64) -> u128 {
    if sorted.is_empty() {
        return 0;
    }
    let idx = ((sorted.len() as f64) * p) as usize;
    sorted[idx.min(sorted.len() - 1)]
}

// ─────────────────────────────────────────────────────────────────────────────
// Core engine
// ─────────────────────────────────────────────────────────────────────────────

/// Fire `config.requests` HTTP requests with up to `config.concurrency` in flight
/// at once.  Progress events are broadcast every 500 ms; a final `Done` (or
/// `Error`) event is sent when all requests finish.
///
/// Using `broadcast::Sender` means the frontend can freely reconnect without
/// crashing the Rust worker.
pub async fn run_blitz(config: BlitzConfig, tx: broadcast::Sender<ProgressEvent>) {
    // Enforce hard safety caps
    let total = config.requests.min(500);
    let concurrency = config.concurrency.min(50);

    let client = Arc::new(Client::builder().timeout(std::time::Duration::from_secs(30)).build().unwrap_or_default());
    let success_count  = Arc::new(AtomicUsize::new(0));
    let failure_count  = Arc::new(AtomicUsize::new(0));
    let completed      = Arc::new(AtomicUsize::new(0));
    let latencies      = Arc::new(Mutex::new(Vec::<u128>::new()));
    let start_times: Arc<Vec<AtomicU64>> = Arc::new(
        (0..total).map(|_| AtomicU64::new(0)).collect(),
    );

    let sem        = Arc::new(Semaphore::new(concurrency));
    let test_start = tokio::time::Instant::now();

    // ── spawn one task per request ──────────────────────────────────────────
    let mut handles = Vec::with_capacity(total);

    for i in 0..total {
        let client        = Arc::clone(&client);
        let success_count = Arc::clone(&success_count);
        let failure_count = Arc::clone(&failure_count);
        let completed     = Arc::clone(&completed);
        let latencies     = Arc::clone(&latencies);
        let start_times   = Arc::clone(&start_times);
        let sem           = Arc::clone(&sem);
        let url           = config.url.clone();
        let method        = config.method.clone();
        let scenario      = config.scenario.clone();

        handles.push(tokio::spawn(async move {
            let _permit = sem.acquire().await.unwrap();

            // record when this request actually starts flying
            let since_epoch = test_start.elapsed().as_nanos() as u64;
            start_times[i].store(since_epoch, Ordering::Relaxed);

            let payload = generate_payload(&scenario);
            let req_start = tokio::time::Instant::now();

            let response = match method {
                HttpMethod::Get => client.get(&url).send().await,
                HttpMethod::Post => {
                    let mut b = client.post(&url).header("Content-Type", "application/json");
                    if let Some(p) = payload {
                        b = b.body(p);
                    }
                    b.send().await
                }
            };

            let duration = req_start.elapsed().as_millis();

            match response {
                Ok(r) if r.status().is_success() => {
                    success_count.fetch_add(1, Ordering::Relaxed);
                    latencies.lock().unwrap().push(duration);
                }
                _ => {
                    failure_count.fetch_add(1, Ordering::Relaxed);
                }
            }

            completed.fetch_add(1, Ordering::Relaxed);
        }));
    }

    // ── progress reporter (every 500 ms) ────────────────────────────────────
    let tx_prog   = tx.clone();
    let comp_ref  = Arc::clone(&completed);
    let succ_ref  = Arc::clone(&success_count);
    let fail_ref  = Arc::clone(&failure_count);
    let lats_ref  = Arc::clone(&latencies);

    let reporter = tokio::spawn(async move {
        loop {
            tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;

            let done  = comp_ref.load(Ordering::Relaxed);
            let succ  = succ_ref.load(Ordering::Relaxed);
            let fail  = fail_ref.load(Ordering::Relaxed);
            let elapsed = test_start.elapsed().as_secs_f64();
            let rps   = if elapsed > 0.0 { done as f64 / elapsed } else { 0.0 };

            let (avg, p95) = {
                let v = lats_ref.lock().unwrap();
                if v.is_empty() {
                    (0.0, 0)
                } else {
                    let avg = v.iter().sum::<u128>() as f64 / v.len() as f64;
                    let mut s = v.clone();
                    s.sort_unstable();
                    (avg, percentile(&s, 0.95))
                }
            };

            // lagged receivers just get dropped – that's fine for progress events
            let _ = tx_prog.send(ProgressEvent::Progress {
                completed: done,
                total,
                successes: succ,
                failures: fail,
                rps,
                avg_latency_ms: avg,
                p95_latency_ms: p95,
            });

            if done >= total {
                break;
            }
        }
    });

    // ── wait for all request tasks ──────────────────────────────────────────
    futures::future::join_all(handles).await;
    reporter.abort();

    // ── final Done event ────────────────────────────────────────────────────
    let elapsed_ms = test_start.elapsed().as_millis() as u64;
    let done  = completed.load(Ordering::Relaxed);
    let succ  = success_count.load(Ordering::Relaxed);
    let fail  = failure_count.load(Ordering::Relaxed);
    let rps   = if elapsed_ms > 0 { done as f64 / (elapsed_ms as f64 / 1000.0) } else { 0.0 };

    let mut final_lats = latencies.lock().unwrap().clone();
    final_lats.sort_unstable();

    let avg_ms = if !final_lats.is_empty() {
        final_lats.iter().sum::<u128>() as f64 / final_lats.len() as f64
    } else { 0.0 };

    // start-time spread (measures thundering-herd quality)
    let mut starts: Vec<u64> = start_times
        .iter()
        .map(|a| a.load(Ordering::Relaxed))
        .filter(|&v| v > 0)
        .collect();
    starts.sort_unstable();
    let spread_us = if starts.len() > 1 {
        (starts.last().unwrap() - starts.first().unwrap()) / 1_000
    } else { 0 };

    let _ = tx.send(ProgressEvent::Done {
        completed: done,
        total,
        successes: succ,
        failures: fail,
        total_time_ms: elapsed_ms,
        rps,
        avg_latency_ms: avg_ms,
        p50_latency_ms: percentile(&final_lats, 0.50),
        p95_latency_ms: percentile(&final_lats, 0.95),
        p99_latency_ms: percentile(&final_lats, 0.99),
        spread_us,
    });
}
