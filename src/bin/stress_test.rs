use reqwest::Client;
use serde_json::json;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::Semaphore;

const BASE_URL: &str = "http://127.0.0.1:8080";
const NUM_CLIENTS: usize = 100;
const TOTAL_REQUESTS: usize = 5_000;
const CONCURRENCY: usize = 500;
const CREDIT_AMOUNT: &str = "10.00";

#[tokio::main]
async fn main() {
    println!("╔══════════════════════════════════════╗");
    println!("║          PREX — STRESS TEST          ║");
    println!("╚══════════════════════════════════════╝");
    println!("  Endpoint    : {BASE_URL}");
    println!("  Requests    : {TOTAL_REQUESTS}");
    println!("  Concurrencia: {CONCURRENCY}");
    println!("  Clientes    : {NUM_CLIENTS}");
    println!();

    let client = Client::builder()
        .timeout(Duration::from_secs(30))
        .build()
        .expect("I don't make client HTTP");

    phase_1_health_check(&client).await;
    let client_ids = phase_2_provision_clients(&client).await;
    let (results, total_duration) = phase_3_run_load(&client, &client_ids).await;
    phase_4_print_summary(&results, total_duration);
}

async fn phase_1_health_check(client: &Client) {
    print!("[Fase 1] Verificando servidor ... ");
    match client.get(BASE_URL).send().await {
        Ok(r) if r.status().is_success() => println!("OK ✓"),
        Ok(r) => {
            println!("status inesperado: {}", r.status());
            std::process::exit(1);
        }
        Err(e) => {
            println!("ERROR: {e}");
            println!("        Asegurate de que el servidor esté corriendo: cargo run");
            std::process::exit(1);
        }
    }
}

// Provisionamiento de clientes de prueba
async fn phase_2_provision_clients(client: &Client) -> Vec<u64> {
    print!("Make {NUM_CLIENTS} client of test ... ");

    let mut ids = Vec::with_capacity(NUM_CLIENTS);

    for i in 0..NUM_CLIENTS {
        let payload = json!({
            "client_name": format!("Stress User {i:04}"),
            "birth_date": "1990-01-01",
            "document_number": format!("STRESS-{i:06}"),
            "country": "AR"
        });

        if let Ok(resp) = client
            .post(format!("{BASE_URL}/new_client"))
            .json(&payload)
            .send()
            .await
        {
            if resp.status().is_success() {
                if let Ok(id) = resp.json::<u64>().await {
                    ids.push(id);
                }
            }
        }
    }

    println!("{}/{NUM_CLIENTS} OK ✓", ids.len());

    if ids.is_empty() {
        eprintln!("Error: Could not create any client. Aborting.");
        std::process::exit(1);
    }

    ids
}

// Generación de carga
struct RequestResult {
    duration: Duration,
    success: bool,
    status: u16,
}

async fn phase_3_run_load(
    client: &Client,
    client_ids: &[u64],
) -> (Vec<RequestResult>, Duration) {
    println!("Request {TOTAL_REQUESTS} concurrents ...");

    let semaphore = Arc::new(Semaphore::new(CONCURRENCY));
    let http = Arc::new(client.clone());
    let ids = Arc::new(client_ids.to_vec());

    let mut handles = Vec::with_capacity(TOTAL_REQUESTS);
    let wall_start = Instant::now();

    for i in 0..TOTAL_REQUESTS {
        let sem = semaphore.clone();
        let http = http.clone();
        let ids = ids.clone();

        handles.push(tokio::spawn(async move {
            let _permit = sem.acquire().await.unwrap();
            let client_id = ids[i % ids.len()];

            let payload = json!({
                "client_id": client_id,
                "credit_amount": CREDIT_AMOUNT
            });

            let req_start = Instant::now();
            let result = http
                .post(format!("{BASE_URL}/new_credit_transaction"))
                .json(&payload)
                .send()
                .await;
            let duration = req_start.elapsed();

            match result {
                Ok(r) => RequestResult {
                    duration,
                    success: r.status().is_success(),
                    status: r.status().as_u16(),
                },
                Err(_) => RequestResult {
                    duration,
                    success: false,
                    status: 0,
                },
            }
        }));
    }

    let mut results = Vec::with_capacity(TOTAL_REQUESTS);
    for h in handles {
        if let Ok(r) = h.await {
            results.push(r);
        }
    }

    let total_duration = wall_start.elapsed();
    println!("        Success in {:.2}s ✓", total_duration.as_secs_f64());

    (results, total_duration)
}

// Reporte de resultados

fn phase_4_print_summary(results: &[RequestResult], total_duration: Duration) {
    let total = results.len();
    let successes = results.iter().filter(|r| r.success).count();
    let failures = total - successes;
    let success_rate = successes as f64 / total as f64 * 100.0;
    let throughput = total as f64 / total_duration.as_secs_f64();

    let mut latencies: Vec<u128> = results.iter().map(|r| r.duration.as_millis()).collect();
    latencies.sort_unstable();
    let avg_ms = latencies.iter().sum::<u128>() as f64 / latencies.len() as f64;

    println!();
    println!("╔══════════════════════════════════════╗");
    println!("║       ASWER OF STRESS TEST           ║");
    println!("╠══════════════════════════════════════╣");
    println!("║  Size                             ║");
    println!("║    Total requests  : {total:<15}║");
    println!("║    Success         : {successes:<8} ({success_rate:.1}%)  ║");
    println!("║    Failures        : {failures:<15}║");
    println!("╠══════════════════════════════════════╣");
    println!("║  Performance                         ║");
    println!("║    Time total      : {:.2}s{:<12}║", total_duration.as_secs_f64(), "");
    println!("║    Throughput      : {throughput:<8.0} req/s     ║");
    println!("╠══════════════════════════════════════╣");
    println!("║  Latency  (ms)                       ║");
    println!("║    Average         : {avg_ms:<15.1}║");
    println!("║    p50             : {:<15}║", percentile(&latencies, 50));
    println!("║    p95             : {:<15}║", percentile(&latencies, 95));
    println!("║    p99             : {:<15}║", percentile(&latencies, 99));
    println!("║    Max             : {:<15}║", latencies.last().unwrap_or(&0));
    println!("╚══════════════════════════════════════╝");

    if failures > 0 {
        println!();
        println!("  Error distribution:");
        let mut counts: HashMap<u16, usize> = HashMap::new();
        for r in results.iter().filter(|r| !r.success) {
            *counts.entry(r.status).or_insert(0) += 1;
        }
        let mut sorted: Vec<_> = counts.into_iter().collect();
        sorted.sort_by_key(|(k, _)| *k);
        for (status, count) in sorted {
            let label = if status == 0 {
                "Timeout/Error on red".to_string()
            } else {
                format!("HTTP {status}")
            };
            println!("    {label}: {count}");
        }
    }
}

fn percentile(sorted: &[u128], p: usize) -> u128 {
    if sorted.is_empty() {
        return 0;
    }
    let idx = (sorted.len() * p / 100).saturating_sub(1).min(sorted.len() - 1);
    sorted[idx]
}
