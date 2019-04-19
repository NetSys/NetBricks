use failure::Error;

pub type Result<T> = ::std::result::Result<T, Error>;

#[macro_export]
macro_rules! error_chain {
    ($error:expr) => {
        error!("{}", $crate::common::errors::string_chain($error))
    };
}

#[macro_export]
macro_rules! warn_chain {
    ($error:expr) => {
        warn!("{}", $crate::common::errors::string_chain($error))
    };
}

/// Read a `failure` `Error` and print out the causes and a backtrace as
/// `log::error`s
pub fn string_chain(e: &Error) -> String {
    let mut error = e.to_string();

    for cause in e.iter_causes() {
        error.push_str(&format!("\nCaused by: {}", cause));
    }

    if let Ok("1") = ::std::env::var("RUST_BACKTRACE")
        .as_ref()
        .map(|s| s.as_str())
    {
        error.push_str(&format!("\nBacktrace:\n{}", e.backtrace()))
    }

    error
}
