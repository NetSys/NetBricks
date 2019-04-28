extern crate netbricks;
use netbricks::common::Result;
use netbricks::config::load_config;

fn main() -> Result<()> {
    let configuration = load_config()?;
    println!("{}", configuration);
    Ok(())
}
