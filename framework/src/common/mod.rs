mod errors;
pub use self::errors::*;

/// Null metadata associated with packets initially.
pub struct EmptyMetadata;

pub fn print_error(e: &Error) {
    println!("Error: {}", e);
    for e in e.iter().skip(1) {
        println!("Cause: {}", e);
    }
    if let Some(backtrace) = e.backtrace() {
        println!("Backtrace: {:?}", backtrace);
    }
}
