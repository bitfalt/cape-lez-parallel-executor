use crate::executor::{
    ExecutionConfig, Executor, OptimisticExecutor, SequentialExecutor, StaticSchedulerExecutor,
};
use crate::metrics::RunMetrics;
use crate::workload::{WorkloadConfig, generator_by_name, workload_names};
use std::fs::{self, OpenOptions};
use std::path::Path;

#[derive(Debug, Clone)]
pub struct BenchmarkOptions {
    pub executor: String,
    pub workload: String,
    pub block_size: usize,
    pub accounts: usize,
    pub threads: usize,
    pub seed: u64,
    pub compute_cost: u64,
    pub repetitions: usize,
    pub output: Option<String>,
    pub skew: f64,
    pub duplicate_nullifier_rate: f64,
}

impl Default for BenchmarkOptions {
    fn default() -> Self {
        Self {
            executor: "all".into(),
            workload: "all".into(),
            block_size: 1000,
            accounts: 1000,
            threads: 1,
            seed: 42,
            compute_cost: 1000,
            repetitions: 1,
            output: None,
            skew: 1.1,
            duplicate_nullifier_rate: 0.15,
        }
    }
}

pub fn executor_names(selection: &str) -> Vec<String> {
    if selection == "all" {
        vec!["sequential".into(), "static".into(), "optimistic".into()]
    } else {
        selection
            .split(',')
            .map(|item| item.trim().to_string())
            .collect()
    }
}

pub fn run_benchmark(options: &BenchmarkOptions) -> Vec<RunMetrics> {
    let mut rows = Vec::new();
    for repetition in 0..options.repetitions {
        for workload_name in workload_names(&options.workload) {
            let generator = generator_by_name(&workload_name);
            let workload_config = WorkloadConfig {
                block_size: options.block_size,
                accounts: options.accounts,
                seed: options.seed + repetition as u64,
                compute_cost: options.compute_cost,
                skew: options.skew,
                duplicate_nullifier_rate: options.duplicate_nullifier_rate,
            };
            let scenario = generator.generate(&workload_config);
            for executor_name in executor_names(&options.executor) {
                let executor = executor_by_name(&executor_name);
                let config = ExecutionConfig {
                    threads: options.threads,
                    fallback_reexecution_rate: 0.75,
                };
                let report = executor.execute(&scenario.initial_state, &scenario.block, &config);
                let mut metrics = report.metrics;
                metrics.run_id = format!(
                    "{}-{}-{}-{}-{}",
                    repetition, workload_name, executor_name, options.threads, options.block_size
                );
                metrics.seed = workload_config.seed;
                metrics.executor = executor_name;
                metrics.workload = scenario.name.clone();
                metrics.block_size = scenario.block.len();
                metrics.num_accounts = scenario.accounts;
                metrics.threads = options.threads;
                metrics.compute_cost = options.compute_cost;
                metrics.skew = options.skew;
                metrics.final_state_hash = report.final_state_hash;
                metrics.finalize_timing();
                rows.push(metrics);
            }
        }
    }

    if let Some(output) = &options.output {
        write_metrics(output, &rows).expect("metrics CSV must be writable");
    }
    rows
}

pub fn run_matrix(
    base: &BenchmarkOptions,
    block_sizes: &[usize],
    thread_counts: &[usize],
) -> Vec<RunMetrics> {
    let mut all_rows = Vec::new();
    for block_size in block_sizes {
        for threads in thread_counts {
            let mut options = base.clone();
            options.block_size = *block_size;
            options.threads = *threads;
            options.output = None;
            all_rows.extend(run_benchmark(&options));
        }
    }
    if let Some(output) = &base.output {
        write_metrics(output, &all_rows).expect("metrics CSV must be writable");
    }
    all_rows
}

pub fn write_metrics(path: impl AsRef<Path>, rows: &[RunMetrics]) -> csv::Result<()> {
    let path = path.as_ref();
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let file_exists = path.exists();
    let file = OpenOptions::new().create(true).append(true).open(path)?;
    let mut writer = csv::WriterBuilder::new()
        .has_headers(!file_exists)
        .from_writer(file);
    for row in rows {
        writer.serialize(row)?;
    }
    writer.flush()?;
    Ok(())
}

pub fn parse_usize_list(input: &str) -> Vec<usize> {
    input
        .split(',')
        .filter_map(|item| item.trim().parse::<usize>().ok())
        .collect()
}

fn executor_by_name(name: &str) -> Box<dyn Executor> {
    match name {
        "sequential" => Box::new(SequentialExecutor),
        "static" => Box::new(StaticSchedulerExecutor),
        "optimistic" => Box::new(OptimisticExecutor),
        other => panic!("unknown executor: {other}"),
    }
}
