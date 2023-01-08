use clap::Parser;
//use futures::executor::block_on;
use ndarray::{Array1, Axis};
use ndarray_stats::{interpolate::Nearest, QuantileExt};
use noisy_float::types::n64;
use polars::prelude::*;
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
    #[arg(short, long, default_value_t = 100)]
    count: u8,

    /// threshold for the 99th percentile in ms. Keep the float notation.
    #[arg(short, long, default_value_t = 1000.0)]
    threshold: f64,

    /// gate is a flag to decide if the tool should throw an error if the 95th percentile is above the threshold
    #[arg(short, long)]
    gate: bool,

    /// verbose mode to print the results of each request
    #[arg(short, long)]
    verbose: bool,

    /// enable load emulation
    #[arg(short, long)]
    emulate_load: bool,

    /// number of requests to emulate
    #[arg(short, long, default_value_t = 100)]
    requests_count: u8,
}

async fn emulate_load(url: &str, n: u8) {
    println!(
        ">> NOT IMPLEMENTED. Emulating load on {} with {} requests",
        url, n
    );
}

#[tokio::main]
async fn main() {
    let args = Args::parse();
    // TODO: need another thread to emulate a load on the server (or call an external command?)
    if args.emulate_load {
        emulate_load(&args.url, args.requests_count).await;
    }
    let n = args.count;
    let url = args.url.as_str();
    let mut ttfb_vec = Vec::new();

    println!(">> Running {} tests on {}", n, url);

    // if verbose mode is disabled print a warning
    if !args.verbose {
        println!(">> Silent mode enabled. Please wait...");
    }

    //let df = DataFrame::default();

    for i in 1..=n {
        let client = Client::new();

        let timer = Instant::now();
        let response = client.get(url).send().await.unwrap();
        let elapsed = timer.elapsed();

        if args.verbose {
            println!(
                "Run {}/{}: TTFB: {:?}ms, Status: {}",
                i,
                n,
                elapsed.as_millis(),
                response.status()
            );
        }
        ttfb_vec.push(elapsed.as_millis() as f64);
        // also push to the polars dataframe
    }

    let s = Series::new("ttfb", ttfb_vec.clone());
    let df = DataFrame::new(vec![s]).unwrap();
    let _df2: DataFrame = df.describe(None);

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
    println!("---");
    println!("Max (Slowest): {}ms", max);
    println!("95th percentile: {}ms", percentile_95);
    println!("Median: {}ms", median);
    println!("---");
    println!("99th percentile: {}ms", percentile_99);
    println!("Mean: {}ms", mean);
    println!("Min: {}ms", min);
    println!("---");

    if args.gate {
        let gate_metric = percentile_95;
        if gate_metric > args.threshold {
            println!(
                "⚠️ 95th percentile is above the threshold of {}ms",
                args.threshold
            );
            println!("💀 Exiting with error code 1");
            std::process::exit(1);
        } else {
            println!(
                "👍 95th percentile is below the threshold of {}ms",
                args.threshold
            );
            std::process::exit(0);
        }
    }
}
