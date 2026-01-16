use pyo3::prelude::*;
use pyo3::types::PyModule;
use std::ffi::CStr;
use std::sync::{Arc, Mutex};
use std::sync::atomic::{AtomicUsize, Ordering};
use tokio::time::Instant;
use indicatif::ProgressBar;
use clap::Parser;

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

    // 1. Setup Python Environment
    let script_content = std::fs::read_to_string("scripts/scenario.py")
        .expect("Missing scripts/scenario.py");
    let binding = [script_content.as_bytes(), &[0]].concat();
    let c_script = CStr::from_bytes_with_nul(binding.as_slice()).unwrap();

    // 2. Setup Shared State
    let success_count = Arc::new(AtomicUsize::new(0));
    let failure_count = Arc::new(AtomicUsize::new(0));
    let latencies = Arc::new(Mutex::new(Vec::new()));
    
    let client = Arc::new(reqwest::Client::new());
    let pb = ProgressBar::new(args.requests as u64);
    let start_time = Instant::now();

    println!("🚀 Blitz-Load starting on {}", args.url);

    let mut handles = vec![];

    for _ in 0..args.requests {
        let client = Arc::clone(&client);
        let success_count = Arc::clone(&success_count);
        let failure_count = Arc::clone(&failure_count);
        let latencies = Arc::clone(&latencies);
        let pb = pb.clone();
        let url = args.url.clone();
        let c_script_owned = c_script.to_owned();

        let handle = tokio::spawn(async move {
            // STEP A: Get dynamic payload from Python Brain
            let payload = Python::with_gil(|py| {
                let activ = PyModule::from_code(py, &c_script_owned, c"scenario.py", c"scripts").unwrap();
                let result = activ.getattr("get_payload").unwrap().call0().unwrap();
                let json_lib = py.import("json").unwrap();
                json_lib.call_method1("dumps", (result,)).unwrap().extract::<String>().unwrap()
            });

            // STEP B: Measure Request Latency
            let req_start = Instant::now();
            let response = client.post(url)
                .header("Content-Type", "application/json")
                .body(payload)
                .send()
                .await;

            let duration = req_start.elapsed().as_millis();

            // STEP C: Record Results
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

        // Control worker pool size
        if handles.len() >= args.concurrency {
            let _ = futures::future::join_all(handles.drain(..)).await;
        }
    }

    let _ = futures::future::join_all(handles).await;
    pb.finish_with_message("Done!");

    // 5. Final Analytics
    let total_time = start_time.elapsed();
    let mut final_latencies = latencies.lock().unwrap();
    final_latencies.sort();

    println!("\n--- 📊 LOAD TEST RESULTS ---");
    println!("Total Requests: {}", args.requests);
    println!("Successes:      {}", success_count.load(Ordering::SeqCst));
    println!("Failures:       {}", failure_count.load(Ordering::SeqCst));
    println!("RPS:            {:.2}", args.requests as f64 / total_time.as_secs_f64());
    
    if !final_latencies.is_empty() {
        let p95 = final_latencies[(final_latencies.len() as f64 * 0.95) as usize];
        let p99 = final_latencies[(final_latencies.len() as f64 * 0.99) as usize];
        println!("p95 Latency:    {}ms", p95);
        println!("p99 Latency:    {}ms", p99);
    }

    Ok(())
}