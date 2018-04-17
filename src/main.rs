#![allow(dead_code)]

use std::fs::File;

pub mod cpu;
pub mod inst;
pub mod cart;
pub mod mem;

fn main() {
    let mut args = std::env::args();
    let cart_path = args.nth(1).unwrap();
    let cart_file = File::open(cart_path.clone()).unwrap();
    let c = cart::Cart::load(cart_file).unwrap();

    println!("Loaded cart {}:", cart_path);
    println!("Name: {}", c.name());
    println!("File Size: {} bytes", c.data.len());
    println!("Cart type: {}", c.type_());
    println!("ROM Size: {} bytes", c.rom_size());
    println!("RAM Size: {} bytes", c.ram_size());

    let mut runner = cpu::Cpu::new(c);
    loop {
        runner.run_cycle();
    }
}