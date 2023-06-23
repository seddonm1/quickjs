extern crate quickjs;

use anyhow::Result;
use clap::Parser;
use quickjs::{QuickJS, TimeLimit};
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

    /// Enable stdout (i.e. console.log) defualt false
    #[arg(long)]
    inherit_stdout: bool,

    /// Enable stderr (i.e. console.error) default false
    #[arg(long)]
    inherit_stderr: bool,

    /// Set runtime memory limit in bytes to restrict unconstrained memory growth
    #[arg(long)]
    memory_limit_bytes: Option<usize>,

    /// Set runtime time limit in nanoseconds
    #[arg(long)]
    time_limit_nanos: Option<u64>,

    /// Set time limit evaluation frequency in nanoseconds. Only used if `time_limit_nanos` is set.
    #[arg(long, default_value_t = 10000000)]
    time_limit_evaluation_frequency_nanos: u64,
}

fn main() -> Result<()> {
    let args = Args::parse();

    let quickjs = QuickJS::try_new(
        args.module,
        args.inherit_stdout,
        args.inherit_stderr,
        args.memory_limit_bytes,
        args.time_limit_nanos.map(|nanos| TimeLimit {
            time_limit: Duration::from_nanos(nanos),
            evaluation_frequency: Duration::from_nanos(args.time_limit_evaluation_frequency_nanos),
        }),
    )?;

    let script = std::fs::read_to_string(args.script)?;
    let data = std::fs::read_to_string(args.data)?;

    let start = Instant::now();
    for i in 0..args.iterations {
        let output = quickjs.try_execute(&script, Some(&data))?;
        println!("{i} {}", output.unwrap_or_else(|| "None".to_string()));
    }

    let duration = start.elapsed();
    println!(
        "elapsed: {:?}\niteration: {:?}",
        duration,
        duration.div_f32(args.iterations as f32)
    );

    Ok(())
}
