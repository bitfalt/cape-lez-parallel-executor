use cape::bench::{BenchmarkOptions, parse_usize_list, run_benchmark, run_matrix};
use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "cape")]
#[command(about = "Conflict-Aware Parallel Executor benchmark prototype")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Benchmark(RunArgs),
    BenchmarkMatrix(MatrixArgs),
    CheckCorrectness(RunArgs),
}

#[derive(Parser, Debug, Clone)]
struct RunArgs {
    #[arg(long, default_value = "all")]
    executor: String,
    #[arg(long, default_value = "token-transfer")]
    workload: String,
    #[arg(long, default_value_t = 1000)]
    block_size: usize,
    #[arg(long, default_value_t = 1000)]
    accounts: usize,
    #[arg(long, default_value_t = 1)]
    threads: usize,
    #[arg(long, default_value_t = 42)]
    seed: u64,
    #[arg(long, default_value_t = 1000)]
    compute_cost: u64,
    #[arg(long, default_value_t = 1)]
    repetitions: usize,
    #[arg(long)]
    output: Option<String>,
    #[arg(long, default_value_t = 1.1)]
    skew: f64,
}

#[derive(Parser, Debug, Clone)]
struct MatrixArgs {
    #[arg(long, default_value = "all")]
    executor: String,
    #[arg(long, default_value = "all")]
    workload: String,
    #[arg(long, default_value = "1000,5000")]
    block_sizes: String,
    #[arg(long, default_value_t = 1000)]
    accounts: usize,
    #[arg(long, default_value = "1,2,4,8")]
    threads: String,
    #[arg(long, default_value_t = 42)]
    seed: u64,
    #[arg(long, default_value_t = 1000)]
    compute_cost: u64,
    #[arg(long, default_value_t = 3)]
    repetitions: usize,
    #[arg(long, default_value = "results/raw/results.csv")]
    output: String,
    #[arg(long, default_value_t = 1.1)]
    skew: f64,
}

fn main() {
    let cli = Cli::parse();
    match cli.command {
        Commands::Benchmark(args) => {
            let options = options_from_run_args(args);
            let rows = run_benchmark(&options);
            print_rows(&rows);
        }
        Commands::BenchmarkMatrix(args) => {
            let block_sizes = parse_usize_list(&args.block_sizes);
            let threads = parse_usize_list(&args.threads);
            let options = BenchmarkOptions {
                executor: args.executor,
                workload: args.workload,
                accounts: args.accounts,
                seed: args.seed,
                compute_cost: args.compute_cost,
                repetitions: args.repetitions,
                output: Some(args.output),
                skew: args.skew,
                ..BenchmarkOptions::default()
            };
            let rows = run_matrix(&options, &block_sizes, &threads);
            print_rows(&rows);
        }
        Commands::CheckCorrectness(args) => {
            let mut options = options_from_run_args(args);
            options.executor = "static,optimistic".into();
            let rows = run_benchmark(&options);
            let failures = rows
                .iter()
                .filter(|row| row.correct_vs_sequential != Some(true))
                .count();
            print_rows(&rows);
            if failures > 0 {
                eprintln!("{failures} correctness failures");
                std::process::exit(1);
            }
        }
    }
}

fn options_from_run_args(args: RunArgs) -> BenchmarkOptions {
    BenchmarkOptions {
        executor: args.executor,
        workload: args.workload,
        block_size: args.block_size,
        accounts: args.accounts,
        threads: args.threads,
        seed: args.seed,
        compute_cost: args.compute_cost,
        repetitions: args.repetitions,
        output: args.output,
        skew: args.skew,
        ..BenchmarkOptions::default()
    }
}

fn print_rows(rows: &[cape::metrics::RunMetrics]) {
    for row in rows {
        println!(
            "{},{},{},{},{},{:.2},correct={:?}",
            row.executor,
            row.workload,
            row.block_size,
            row.threads,
            row.seed,
            row.throughput_tps,
            row.correct_vs_sequential
        );
    }
}
