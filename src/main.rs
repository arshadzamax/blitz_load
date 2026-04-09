use clap::Parser;
use indicatif::ProgressBar;
use pyo3::prelude::*;
use pyo3::types::PyModule;
use std::ffi::CStr;

use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};

use tokio::net::UdpSocket;
use tokio::sync::Notify;
use tokio::time::{sleep, Duration, Instant};

#[derive(Parser, Debug)]
#[command(author, version, about = "High-performance Hybrid Load Tester")]
struct Args {
    #[arg(short, long)]
    url: String,

    #[arg(short, long, default_value_t = 10)]
    concurrency: usize,

    #[arg(short, long, default_value_t = 100)]
    requests: usize,
}

#[tokio::main]
async fn main() -> PyResult<()> {
    let args = Args::parse();

    // ------------------------------------------------------------
    // Load Python scenario script
    // ------------------------------------------------------------
    let script_content =
        std::fs::read_to_string("scripts/scenario.py")
            .expect("Missing scripts/scenario.py");

    let binding = [script_content.as_bytes(), &[0]].concat();
    let c_script = CStr::from_bytes_with_nul(binding.as_slice()).unwrap();

    // ------------------------------------------------------------
    // Shared metrics
    // ------------------------------------------------------------
    let success_count = Arc::new(AtomicUsize::new(0));
    let failure_count = Arc::new(AtomicUsize::new(0));

    let latencies = Arc::new(Mutex::new(Vec::<u128>::new()));

    // Lock-free timestamp storage
    let start_times = Arc::new(
        (0..args.requests)
            .map(|_| AtomicU64::new(0))
            .collect::<Vec<_>>(),
    );

    // Trigger synchronization primitive
    let trigger = Arc::new(Notify::new());

    let client = Arc::new(reqwest::Client::new());
    let pb = ProgressBar::new(args.requests as u64);

    let start_time = Instant::now();

    println!("🚀 Blitz-Load starting on {}", args.url);

    let mut handles = vec![];

    // ------------------------------------------------------------
    // Spawn request workers
    // ------------------------------------------------------------
    for i in 0..args.requests {
        let client = Arc::clone(&client);
        let success_count = Arc::clone(&success_count);
        let failure_count = Arc::clone(&failure_count);
        let latencies = Arc::clone(&latencies);
        let start_times = Arc::clone(&start_times);
        let trigger = Arc::clone(&trigger);

        let pb = pb.clone();
        let url = args.url.clone();
        let c_script_owned = c_script.to_owned();

        let handle = tokio::spawn(async move {
            // =====================================================
            // PHASE 1 — PREPARE REQUEST
            // =====================================================
            let payload = Python::with_gil(|py| {
                let module = PyModule::from_code(
                    py,
                    &c_script_owned,
                    c"scenario.py",
                    c"scripts",
                )
                .unwrap();

                let result = module
                    .getattr("get_payload")
                    .unwrap()
                    .call0()
                    .unwrap();

                let json = py.import("json").unwrap();

                json.call_method1("dumps", (result,))
                    .unwrap()
                    .extract::<String>()
                    .unwrap()
            });

            // =====================================================
            // WAIT FOR TRIGGER
            // =====================================================
            trigger.notified().await;

            // =====================================================
            // PHASE 2 — SEND REQUEST
            // =====================================================
            let req_start = Instant::now();

            let since_beginning =
                req_start.duration_since(start_time).as_nanos();

            start_times[i].store(since_beginning as u64, Ordering::Relaxed);

            let response = client
                .post(url)
                .header("Content-Type", "application/json")
                .body(payload)
                .send()
                .await;

            let duration = req_start.elapsed().as_millis();

            match response {
                Ok(resp) if resp.status().is_success() => {
                    success_count.fetch_add(1, Ordering::SeqCst);
                    latencies.lock().unwrap().push(duration);
                }
                _ => {
                    failure_count.fetch_add(1, Ordering::SeqCst);
                }
            }

            pb.inc(1);
        });

        handles.push(handle);
    }

    // ------------------------------------------------------------
    // UDP LISTENER
    // ------------------------------------------------------------
    let trigger_socket = UdpSocket::bind("127.0.0.1:9000").await.unwrap();

    println!("Waiting for UDP trigger on 127.0.0.1:9000...");

    // Auto trigger after short delay
    tokio::spawn(async {
        sleep(Duration::from_millis(200)).await;

        let sender = UdpSocket::bind("127.0.0.1:0").await.unwrap();

        let _ = sender.send_to(&[1], "127.0.0.1:9000").await;
    });

    let mut buf = [0u8; 1];

    trigger_socket.recv(&mut buf).await.unwrap();

    println!("Trigger received! Releasing workers...");

    trigger.notify_waiters();

    // Wait for workers
    let _ = futures::future::join_all(handles).await;

    pb.finish_with_message("Done!");

    // ------------------------------------------------------------
    // Synchronization analysis
    // ------------------------------------------------------------
    let mut starts: Vec<u64> = start_times
        .iter()
        .map(|x| x.load(Ordering::Relaxed))
        .filter(|x| *x > 0)
        .collect();

    if starts.len() > 1 {
        starts.sort();

        let first = starts.first().unwrap();
        let last = starts.last().unwrap();

        let spread_ns = last - first;

        println!("\n--- ⏱ START TIME ANALYSIS ---");
        println!("Earliest send: {} ns", first);
        println!("Latest send:   {} ns", last);
        println!("Spread:        {} µs", spread_ns / 1_000);
    }

    // ------------------------------------------------------------
    // Latency statistics
    // ------------------------------------------------------------
    let total_time = start_time.elapsed();

    let mut final_latencies = latencies.lock().unwrap();
    final_latencies.sort();

    println!("\n--- 📊 LOAD TEST RESULTS ---");
    println!("Total Requests: {}", args.requests);
    println!("Successes:      {}", success_count.load(Ordering::SeqCst));
    println!("Failures:       {}", failure_count.load(Ordering::SeqCst));

    println!(
        "RPS:            {:.2}",
        args.requests as f64 / total_time.as_secs_f64()
    );

    if !final_latencies.is_empty() {
        let p95 =
            final_latencies[(final_latencies.len() as f64 * 0.95) as usize];

        let p99 =
            final_latencies[(final_latencies.len() as f64 * 0.99) as usize];

        println!("p95 Latency:    {}ms", p95);
        println!("p99 Latency:    {}ms", p99);
    }

    Ok(())
}