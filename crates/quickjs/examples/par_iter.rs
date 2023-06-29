extern crate quickjs;

use anyhow::Result;
use clap::Parser;
use quickjs::{QuickJS, TimeLimit};
use rayon::prelude::*;
use std::{
    path::PathBuf,
    time::{Duration, Instant},
};

/// Simple program to demonstr
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Path to the wasm module
    #[arg(long)]
    module: Option<PathBuf>,

    /// Path to the input script
    #[arg(long, default_value = "track_points.js")]
    script: PathBuf,

    /// Path to the data json object
    #[arg(long, default_value = "track_points.json")]
    data: PathBuf,

    /// Number of iterations to execute
    #[arg(long, default_value_t = 1000)]
    iterations: usize,

    /// Enable stdout (i.e. console.log) default false
    #[arg(long)]
    inherit_stdout: bool,

    /// Enable stderr (i.e. console.error) default false
    #[arg(long)]
    inherit_stderr: bool,

    /// Set memory limit in bytes to restrict unconstrained memory growth
    #[arg(long)]
    memory_limit_bytes: Option<u32>,

    /// Set time limit in microseconds to restrict runtime
    #[arg(long)]
    time_limit_micros: Option<u64>,

    /// Set time limit evaluation interval. only used if `time_limit_micros` is set.
    #[arg(long)]
    time_limit_evaluation_interval_micros: Option<u64>,
}

fn main() -> Result<()> {
    let args = Args::parse();

    let quickjs = QuickJS::try_new(
        args.module,
        args.inherit_stdout,
        args.inherit_stderr,
        args.memory_limit_bytes,
        args.time_limit_micros.map(|limit| {
            let mut limit = TimeLimit::new(Duration::from_micros(limit));
            if let Some(evaluation_interval) = args.time_limit_evaluation_interval_micros {
                limit.evaluation_interval = Duration::from_micros(evaluation_interval);
            }
            limit
        }),
    )?;

    let script = std::fs::read_to_string(args.script)?;
    let data = std::fs::read_to_string(args.data)?;

    let start = Instant::now();

    (0..args.iterations)
        .collect::<Vec<_>>()
        .chunks(args.iterations / num_cpus::get())
        .collect::<Vec<_>>()
        .into_par_iter()
        .map(|chunk| {
            chunk
                .iter()
                .map(|i| {
                    let output = quickjs.try_execute(&script, Some(&data))?;
                    println!("{i} {}", output.unwrap_or_else(|| "None".to_string()));
                    Ok(())
                })
                .collect::<Result<Vec<_>>>()
        })
        .collect::<Result<Vec<_>>>()?;

    let duration = start.elapsed();
    println!(
        "elapsed: {:?}\niteration: {:?}",
        duration,
        duration.div_f32(args.iterations as f32)
    );

    Ok(())
}
