use crate::{FileResult, Options, ParseResult};
use crossbeam::channel::Receiver;
use serde::Serialize;
use std::cmp::Reverse;
use std::fs::write;
use std::path::PathBuf;

#[derive(Debug, Default, Clone, Serialize)]
struct ResultCollection {
    successes: Vec<PathBuf>,
    failures: Vec<Failure>,
    panics: Vec<Panic>,
}

#[derive(Debug, Clone, Serialize)]
struct Failure {
    count: u64,
    path: PathBuf,
    error: String,
}

#[derive(Debug, Clone, Serialize)]
struct Panic {
    path: PathBuf,
    cause: String,
}

pub fn receive_results(options: &Options, result_source: &Receiver<FileResult>) {
    let mut results = ResultCollection::default();

    while let Ok(result) = result_source.recv() {
        match result.result {
            ParseResult::Success => results.successes.push(result.path),
            ParseResult::Errors { count, error } => results.failures.push(Failure {
                error: error.to_string(),
                count,
                path: result.path,
            }),
            ParseResult::Panic { cause } => results.panics.push(Panic {
                cause,
                path: result.path,
            }),
            ParseResult::Finish => break,
        }
    }

    results.failures.retain(|v| v.count != 0);

    results
        .failures
        .sort_by_cached_key(|v| Reverse((v.count, v.path.clone())));

    println!("Panics: {}", results.panics.len());
    println!("Failures: {}", results.failures.len());
    println!("Successes: {}", results.successes.len());

    if let Some(output) = &options.output {
        write(output, serde_json::to_string_pretty(&results.failures).unwrap()).unwrap();
    }
}