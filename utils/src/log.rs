
use flexi_logger::{Duplicate, FileSpec, Logger, LoggerHandle, with_thread, WriteMode};
use super::error::*;

///
/// Macros to write to the backing file logger.
///
pub use log::{trace as trace, debug as debug, info as info, warn as warn, error as error};

///
/// Initializes the logstream to write to the given file.
///
pub fn initialize (path: & str, filename: & str, spec: & str) -> Result<LoggerHandle>
{
    let file_spec = FileSpec::default()
        .directory(path)
        .basename(filename)
        .use_timestamp(true)
        .suffix("log");

    let logger = Logger::try_with_str(spec)?
        .log_to_file(file_spec)
        .duplicate_to_stderr(Duplicate::Info)
        .write_mode(WriteMode::Direct)
        .format_for_files(with_thread)
        .start()?;

    info!("Logging initialization complete.");

    Ok(logger)
}

