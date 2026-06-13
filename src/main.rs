use std::env;
use std::fs;
use std::process;

use crqa::{build_report, parse_args, print_usage, run, run_ai_review};

fn main() {
    let args: Vec<String> = env::args().collect();
    let program = args.first().map(String::as_str).unwrap_or("crqa");

    let config = match parse_args(&args[1..]) {
        Ok(config) => config,
        Err(message) => {
            eprintln!("{message}");
            eprintln!();
            print_usage(program);
            process::exit(2);
        }
    };

    if config.help {
        print_usage(program);
        return;
    }

    if config.targets.is_empty() {
        print_usage(program);
        process::exit(1);
    }

    match run(&config) {
        Ok(mut result) => {
            if config.ai_review {
                match run_ai_review(&result) {
                    Ok(ai_review) => result.ai_review = Some(ai_review),
                    Err(err) => {
                        eprintln!("ai review error: {err}");
                        process::exit(2);
                    }
                }
            }

            let report = build_report(&result, config.yaml, config.quiet);
            if let Some(path) = &config.output {
                if let Err(err) = fs::write(path, report) {
                    eprintln!("error: failed to write report to {}: {err}", path.display());
                    process::exit(2);
                }
            } else {
                println!("{report}");
            }
            if result.summary.errors > 0 {
                process::exit(1);
            }
        }
        Err(err) => {
            eprintln!("error: {err}");
            process::exit(2);
        }
    }
}
