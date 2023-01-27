extern crate quickjs;

use anyhow::Result;
use clap::Parser;
use quickjs::QuickJS;
use rayon::prelude::*;
use std::{path::PathBuf, time::Instant};

/// Simple program to demonstr
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Path to the wasm module
    #[arg(short, long)]
    module: Option<PathBuf>,

    /// Path to the input script
    #[arg(short, long)]
    script: Option<PathBuf>,

    /// Path to the data json object
    #[arg(short, long)]
    data: Option<PathBuf>,

    /// Number of iterations to execute
    #[arg(short, long, default_value_t = 1000)]
    iterations: usize,

    /// Enable stdout (i.e. console.log) default false
    #[arg(short, long)]
    inherit_stdout: bool,

    /// Enable stderr (i.e. console.error) default false
    #[arg(short, long)]
    inherit_stderr: bool,
}

fn main() -> Result<()> {
    let args = Args::parse();

    let quickjs = match args.module {
        Some(path) => QuickJS::try_from(path)?,
        None => QuickJS::default(),
    };

    let script = match args.script {
        Some(path) => std::fs::read_to_string(path)?,
        None => include_str!("../../../track_points.js").to_string(),
    };

    let data = match args.data {
        Some(path) => std::fs::read_to_string(path)?,
        None => include_str!("../../../track_points.json").to_string(),
    };

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
                    let output = quickjs.try_execute(
                        &script,
                        Some(&data),
                        args.inherit_stdout,
                        args.inherit_stderr,
                    )?;
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
