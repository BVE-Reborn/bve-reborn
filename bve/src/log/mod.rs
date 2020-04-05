use fern::FormatCallback;
use log::{LevelFilter, Record};
use nom::lib::std::fmt::Arguments;
use std::path::PathBuf;

#[macro_export]
macro_rules! panic_log {
    ($($t:tt)*) => {
        let formatted = ::std::format!($($t)*);
        ::log::error!("{}", formatted);
        ::std::panic!("{}", formatted);
    };
}

pub fn log_formatter(out: FormatCallback<'_>, message: &Arguments<'_>, record: &Record<'_>) {
    out.finish(format_args!(
        "{}[{}][{}] {}",
        chrono::Local::now().format("[%H:%M:%S%.3f]"),
        record.level(),
        record.target(),
        message
    ));
}

pub fn enable_logger(file: &Option<PathBuf>, quiet: bool, debug: bool, trace: bool) {
    let filter = if trace {
        LevelFilter::Trace
    } else if debug {
        LevelFilter::Debug
    } else if quiet {
        LevelFilter::Warn
    } else {
        LevelFilter::Info
    };

    let output: fern::Output = if let Some(file) = &file {
        fern::log_file(file).expect("Unable to open log file").into()
    } else {
        fern::Dispatch::new()
            .filter(|m| m.level() > LevelFilter::Warn)
            .chain(std::io::stdout())
            .into()
    };

    fern::Dispatch::new()
        .format(log_formatter)
        .level(filter)
        .chain(fern::Dispatch::new().level(LevelFilter::Warn).chain(std::io::stderr()))
        .chain(output)
        .apply()
        .expect("Unable to set default logger");
}
