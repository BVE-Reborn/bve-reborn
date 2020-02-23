#![allow(clippy::mem_forget)]

use crate::{FileKind, FileResult, Options, ParseResult};
use crossbeam::channel::Receiver;
use serde::Serialize;
use std::cmp::Reverse;
use std::collections::HashMap;
use std::fs::write;
use std::path::PathBuf;

#[derive(Debug, Default, Clone, Serialize)]
struct ResultCollection {
    file_types: HashMap<FileKind, SingleFileCollection>,
}

#[derive(Debug, Default, Clone, Serialize)]
struct SingleFileCollection {
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

pub fn receive_results(options: &Options, result_source: Receiver<FileResult>) {
    let mut results = ResultCollection::default();

    while let Ok(result) = result_source.recv() {
        let single_file_result = results.file_types.entry(result.kind).or_default();
        match result.result {
            ParseResult::Success => single_file_result.successes.push(result.path),
            ParseResult::Errors { count, error } => single_file_result.failures.push(Failure {
                error: error.to_string(),
                count,
                path: result.path,
            }),
            ParseResult::Panic { cause } => single_file_result.panics.push(Panic {
                cause,
                path: result.path,
            }),
            ParseResult::Finish => {
                std::mem::forget(result_source); // We're finishing, we don't actually care about if this is cleaned up, and this prevents a out-of-time panic cascade 
                break;
            }
        }
    }

    let (panics, failures, successes) = results
        .file_types
        .values_mut()
        .map(|single| {
            single
                .failures
                .sort_by_cached_key(|v| Reverse((v.count, v.path.clone())));

            (single.panics.len(), single.failures.len(), single.successes.len())
        })
        .fold((0, 0, 0), |(acc_p, acc_f, acc_s), (p, f, s)| {
            (acc_p + p, acc_f + f, acc_s + s)
        });

    println!("Panics: {}", panics);
    println!("Failures: {}", failures);
    println!("Successes: {}", successes);

    if let Some(output) = &options.output {
        write(output, serde_json::to_string_pretty(&results).unwrap()).unwrap();
    }
}
