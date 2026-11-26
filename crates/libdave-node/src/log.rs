use napi::bindgen_prelude::*;
use napi::threadsafe_function::ThreadsafeFunctionCallMode;
use napi_derive::napi;

/// Severity of a libdave log message.
#[napi]
pub enum LogSeverity {
    Verbose = 0,
    Info = 1,
    Warning = 2,
    Error = 3,
}

#[napi(object)]
pub struct LogEntry {
    pub severity: LogSeverity,
    pub file: String,
    pub line: i32,
    pub message: String,
}

/// Routes libdave's process-wide log output to `callback`; pass `null` to remove it.
/// Messages are delivered asynchronously and do not keep the event loop alive.
#[napi(ts_args_type = "callback?: ((entry: LogEntry) => void) | null")]
pub fn set_log_sink(callback: Option<Function<'_, LogEntry, ()>>) -> Result<()> {
    let Some(callback) = callback else {
        libdave::set_log_sink(None);
        return Ok(());
    };
    let sink = callback
        .build_threadsafe_function()
        .weak::<true>()
        .build()?;
    libdave::set_log_sink(Some(Box::new(move |severity, file, line, message| {
        let severity = match severity {
            libdave::LogSeverity::Verbose => LogSeverity::Verbose,
            libdave::LogSeverity::Info => LogSeverity::Info,
            libdave::LogSeverity::Warning => LogSeverity::Warning,
            libdave::LogSeverity::Error => LogSeverity::Error,
        };
        sink.call(
            LogEntry {
                severity,
                file: file.to_owned(),
                line,
                message: message.to_owned(),
            },
            ThreadsafeFunctionCallMode::NonBlocking,
        );
    })));
    Ok(())
}
