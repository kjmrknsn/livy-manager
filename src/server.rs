use std::env::Args;

pub fn run<T: Iterator>(mut args: T) -> Result<(), String> {
    println!("hi");
    Ok(())
}
