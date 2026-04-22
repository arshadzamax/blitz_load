use blitz_load::engine::{BlitzConfig, HttpMethod, ProgressEvent, ScenarioKind};
use clap::Parser;
use indicatif::{ProgressBar, ProgressStyle};
use tokio::sync::broadcast;

#[derive(Parser, Debug)]
#[command(author, version, about = "High-performance Hybrid Load Tester")]
struct Args {
    #[arg(short, long)]
    url: String,

    #[arg(short, long, default_value_t = 10)]
    concurrency: usize,

    #[arg(short, long, default_value_t = 100)]
    requests: usize,

    #[arg(short = 'm', long, default_value = "post")]
    method: String,

    #[arg(short = 's', long, default_value = "user_registration")]
    scenario: String,

    #[arg(short = 't', long, default_value_t = 30000)]
    timeout_ms: u64,

    #[arg(short = 'l', long)]
    sla_ms: Option<u64>,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    let method = match args.method.to_lowercase().as_str() {
        "get" => HttpMethod::Get,
        _ => HttpMethod::Post,
    };

    let scenario = match args.scenario.as_str() {
        "simple_get" => ScenarioKind::SimpleGet,
        _ => ScenarioKind::UserRegistration, // Removed CustomJson from CLI for simplicity, as it needs body
    };

    let config = BlitzConfig {
        url: args.url.clone(),
        requests: args.requests,
        concurrency: args.concurrency,
        method,
        scenario,
        timeout_ms: args.timeout_ms,
        sla_ms: args.sla_ms,
    };

    println!("🚀 Blitz-Load starting on {}", args.url);

    let (tx, mut rx) = broadcast::channel(512);

    let pb = ProgressBar::new(args.requests as u64);
    pb.set_style(
        ProgressStyle::default_bar()
            .template("[{elapsed_precise}] {bar:40.cyan/blue} {pos:>7}/{len:7} {msg}")
            .unwrap()
            .progress_chars("=>-"),
    );

    // Spawn the engine
    let engine_handle = tokio::spawn(async move {
        blitz_load::engine::run_blitz(config, tx).await;
    });

    // Listen to events and update the progress bar
    while let Ok(event) = rx.recv().await {
        match event {
            ProgressEvent::Progress {
                completed,
                rps,
                avg_latency_ms,
                p95_latency_ms,
                ..
            } => {
                pb.set_position(completed as u64);
                pb.set_message(format!(
                    "({:.1} rps, {:.1}ms avg, {}ms p95)",
                    rps, avg_latency_ms, p95_latency_ms
                ));
            }
            ProgressEvent::Done {
                completed,
                successes,
                failures,
                sla_breaches,
                total_time_ms,
                rps,
                avg_latency_ms,
                p50_latency_ms,
                p95_latency_ms,
                p99_latency_ms,
                spread_us,
                ..
            } => {
                pb.finish_with_message("Done!");

                println!("\n--- ⏱ START TIME ANALYSIS ---");
                println!("Spread:        {} µs", spread_us);

                println!("\n--- 📊 LOAD TEST RESULTS ---");
                println!("Total Requests: {}", completed);
                println!("Successes:      {}", successes);
                println!("Failures:       {}", failures);
                if args.sla_ms.is_some() {
                    println!("SLA Breaches:   {}", sla_breaches);
                }
                println!("RPS:            {:.2}", rps);
                println!("Avg Latency:    {:.2}ms", avg_latency_ms);
                println!("p50 Latency:    {}ms", p50_latency_ms);
                println!("p95 Latency:    {}ms", p95_latency_ms);
                println!("p99 Latency:    {}ms", p99_latency_ms);
                println!("Total Time:     {:.2}s", total_time_ms as f64 / 1000.0);
                
                break; // Exit the listener loop
            }
            ProgressEvent::Error { message } => {
                pb.finish_with_message(format!("Error: {}", message));
                break;
            }
        }
    }

    let _ = engine_handle.await;

    Ok(())
}