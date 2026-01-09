//! Compute Job
//!
//! Heavy computational tasks that can run for extended periods.
//! Supports various computation types with progress tracking.

use rayon::prelude::*;
use serde::{Deserialize, Serialize};
use std::time::Instant;
use tracing::{debug, info};

use crate::domain::models::{Job, JobResult};
use crate::worker::WorkerState;

#[derive(Debug, Deserialize)]
struct ComputePayload {
    /// Type of computation
    compute_type: ComputeType,
    /// Input parameters
    params: serde_json::Value,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
enum ComputeType {
    /// Matrix multiplication
    MatrixMultiply,
    /// Prime number generation
    PrimeGeneration,
    /// Monte Carlo simulation
    MonteCarlo,
    /// Data aggregation
    Aggregate,
    /// Custom computation
    Custom,
}

#[derive(Debug, Serialize)]
struct ComputeResult {
    compute_type: String,
    duration_ms: u64,
    output: serde_json::Value,
}

pub async fn process(job: &Job, state: &WorkerState) -> anyhow::Result<JobResult> {
    let payload: ComputePayload = serde_json::from_value(job.payload.clone())?;
    
    info!(compute_type = ?payload.compute_type, "Starting computation");
    
    let start = Instant::now();
    
    // Run computation on blocking thread pool
    let output = tokio::task::spawn_blocking(move || {
        match payload.compute_type {
            ComputeType::MatrixMultiply => matrix_multiply(&payload.params),
            ComputeType::PrimeGeneration => prime_generation(&payload.params),
            ComputeType::MonteCarlo => monte_carlo(&payload.params),
            ComputeType::Aggregate => aggregate(&payload.params),
            ComputeType::Custom => custom_compute(&payload.params),
        }
    })
    .await??;

    let duration_ms = start.elapsed().as_millis() as u64;
    
    info!(duration_ms, "Computation complete");

    Ok(JobResult {
        output: serde_json::to_value(ComputeResult {
            compute_type: format!("{:?}", payload.compute_type),
            duration_ms,
            output,
        })?,
    })
}

/// Matrix multiplication (example heavy computation)
fn matrix_multiply(params: &serde_json::Value) -> anyhow::Result<serde_json::Value> {
    let size: usize = params.get("size")
        .and_then(|v| v.as_u64())
        .unwrap_or(100) as usize;

    // Generate random matrices
    let a: Vec<Vec<f64>> = (0..size)
        .map(|_| (0..size).map(|_| rand::random::<f64>()).collect())
        .collect();
    
    let b: Vec<Vec<f64>> = (0..size)
        .map(|_| (0..size).map(|_| rand::random::<f64>()).collect())
        .collect();

    // Parallel matrix multiplication
    let result: Vec<Vec<f64>> = (0..size)
        .into_par_iter()
        .map(|i| {
            (0..size)
                .map(|j| {
                    (0..size).map(|k| a[i][k] * b[k][j]).sum()
                })
                .collect()
        })
        .collect();

    // Return summary (not full matrix)
    let sum: f64 = result.par_iter().flatten().sum();
    
    Ok(serde_json::json!({
        "size": size,
        "sum": sum,
        "mean": sum / (size * size) as f64
    }))
}

/// Prime number generation
fn prime_generation(params: &serde_json::Value) -> anyhow::Result<serde_json::Value> {
    let limit: usize = params.get("limit")
        .and_then(|v| v.as_u64())
        .unwrap_or(100_000) as usize;

    // Sieve of Eratosthenes (parallelized where possible)
    let mut sieve = vec![true; limit + 1];
    sieve[0] = false;
    sieve[1] = false;

    for i in 2..=((limit as f64).sqrt() as usize) {
        if sieve[i] {
            let mut j = i * i;
            while j <= limit {
                sieve[j] = false;
                j += i;
            }
        }
    }

    let primes: Vec<usize> = sieve.par_iter()
        .enumerate()
        .filter(|(_, &is_prime)| is_prime)
        .map(|(i, _)| i)
        .collect();

    let count = primes.len();
    let largest = primes.last().copied();

    Ok(serde_json::json!({
        "limit": limit,
        "count": count,
        "largest": largest,
        "sample": primes.iter().take(10).collect::<Vec<_>>()
    }))
}

/// Monte Carlo simulation
fn monte_carlo(params: &serde_json::Value) -> anyhow::Result<serde_json::Value> {
    let iterations: usize = params.get("iterations")
        .and_then(|v| v.as_u64())
        .unwrap_or(1_000_000) as usize;

    // Parallel Monte Carlo for π estimation
    let inside: usize = (0..iterations)
        .into_par_iter()
        .filter(|_| {
            let x: f64 = rand::random();
            let y: f64 = rand::random();
            x * x + y * y <= 1.0
        })
        .count();

    let pi_estimate = 4.0 * (inside as f64) / (iterations as f64);

    Ok(serde_json::json!({
        "iterations": iterations,
        "inside_circle": inside,
        "pi_estimate": pi_estimate,
        "pi_actual": std::f64::consts::PI,
        "error": (pi_estimate - std::f64::consts::PI).abs()
    }))
}

/// Data aggregation
fn aggregate(params: &serde_json::Value) -> anyhow::Result<serde_json::Value> {
    let data = params.get("data")
        .and_then(|v| v.as_array())
        .ok_or_else(|| anyhow::anyhow!("Missing 'data' array"))?;

    let numbers: Vec<f64> = data.iter()
        .filter_map(|v| v.as_f64())
        .collect();

    if numbers.is_empty() {
        anyhow::bail!("No numeric data to aggregate");
    }

    let sum: f64 = numbers.par_iter().sum();
    let count = numbers.len();
    let mean = sum / count as f64;
    
    let variance: f64 = numbers.par_iter()
        .map(|x| (x - mean).powi(2))
        .sum::<f64>() / count as f64;
    
    let std_dev = variance.sqrt();
    
    let mut sorted = numbers.clone();
    sorted.sort_by(|a, b| a.partial_cmp(b).unwrap());
    
    let median = if count % 2 == 0 {
        (sorted[count / 2 - 1] + sorted[count / 2]) / 2.0
    } else {
        sorted[count / 2]
    };

    Ok(serde_json::json!({
        "count": count,
        "sum": sum,
        "mean": mean,
        "median": median,
        "std_dev": std_dev,
        "min": sorted.first(),
        "max": sorted.last()
    }))
}

/// Custom computation (placeholder for user-defined logic)
fn custom_compute(params: &serde_json::Value) -> anyhow::Result<serde_json::Value> {
    // This would be extended based on specific needs
    Ok(serde_json::json!({
        "status": "custom_compute_placeholder",
        "params": params
    }))
}

// Add rand dependency for random number generation
use rand;
