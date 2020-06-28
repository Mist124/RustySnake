#![allow(dead_code)]
mod lib;

extern crate crossterm;

fn main() -> std::io::Result<()> {
    lib::game()?;
    Ok(())
}
