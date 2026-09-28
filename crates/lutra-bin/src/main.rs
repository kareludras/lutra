mod genbook;

use lutra_engine::{BENCH_DEPTH, bench};
use std::io::{stdin, stdout};

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    match args.first().map(String::as_str) {
        // `lutra bench [depth]`: the conventional engine bench entry point
        // (OpenBench and similar tools run it and parse the last line).
        Some("bench") => {
            let depth = args
                .get(1)
                .and_then(|d| d.parse().ok())
                .unwrap_or(BENCH_DEPTH);
            let result = bench(depth);
            println!("{} nodes {} nps", result.nodes, result.nps());
        }
        Some("genbook") => {
            if let Err(e) = genbook::run(&args[1..], &mut stdout().lock()) {
                eprintln!("{e}");
                std::process::exit(2);
            }
        }
        _ => {
            let stdin = stdin();
            let stdout = stdout();
            lutra_uci::run(stdin.lock(), stdout.lock());
        }
    }
}
