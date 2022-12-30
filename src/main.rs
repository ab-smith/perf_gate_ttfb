use clap::Parser;
use ndarray::{Array1, Axis};
use ndarray_stats::{interpolate::Nearest, QuantileExt};
use noisy_float::types::n64;
use reqwest::Client;
use std::time::Instant;
use tokio;

/// CLI tool to measure the TTFB (Time To First Byte) of a given URL and establish a blocking gate if needed
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// target URL
    #[arg(short, long)]
    url: String,

    /// Number of tests to run
    #[arg(short, long, default_value_t = 20)]
    number: u8,

    /// threshold for the 99th percentile in ms. Keep the float notation.
    #[arg(short, long, default_value_t = 2000.0)]
    threshold: f64,

    /// gate is a flag to decide if the tool should throw an error if the 95th percentile is above the threshold
    #[arg(short, long)]
    gate: bool,
}

#[tokio::main]
async fn main() {
    let args = Args::parse();
    // Create a new reqwest client

    let n = args.number;
    let url = args.url.as_str();
    let mut ttfb_vec = Vec::new();

    println!(">> Running {} tests on {}", n, url);

    for _ in 1..=n {
        let client = Client::new();

        // Specify the URL you want to measure the TTFB for

        // Start a timer
        let timer = Instant::now();

        // Send an HTTP GET request to the URL
        let response = client.get(url).send().await.unwrap();

        // Calculate the elapsed time
        let elapsed = timer.elapsed();

        // Print the elapsed time and the status code of the response
        println!(
            "TTFB: {:?}ms, Status: {}",
            elapsed.as_millis(),
            response.status()
        );
        ttfb_vec.push(elapsed.as_millis() as f64);
    }

    let mut my_array: Array1<f64> = Array1::try_from(ttfb_vec).unwrap();

    // I had to switch the order since the quantiles need to borrow a mutable array

    let median = my_array
        .quantile_axis_skipnan_mut(Axis(0), n64(0.5), &Nearest)
        .unwrap()
        .into_scalar();
    let percentile_95 = my_array
        .quantile_axis_skipnan_mut(Axis(0), n64(0.95), &Nearest)
        .unwrap()
        .into_scalar();
    let percentile_99 = my_array
        .quantile_axis_skipnan_mut(Axis(0), n64(0.99), &Nearest)
        .unwrap()
        .into_scalar();

    let mean = my_array.mean().unwrap();
    let min = my_array.min().unwrap();
    let max = my_array.max().unwrap();

    println!(">> Results for {} tests on {}", n, url);
    println!("Max (Slowest): {}ms", max);
    println!("99th percentile: {}ms", percentile_99);
    println!("95th percentile: {}ms", percentile_95);
    println!("Median: {}ms", median);
    println!("Mean: {}ms", mean);
    println!("Min: {}ms", min);

    if percentile_95 > args.threshold {
        println!(
            "⚠️ 95th percentile is above the threshold of {}ms",
            args.threshold
        );
        if args.gate {
            println!("💀 Exiting with error code 1");
            std::process::exit(1);
        }
    }
}
