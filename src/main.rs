mod rand;
mod fontset;
mod cpu;

use rand::Rand;
use cpu::Chip8;

fn main() {
    println!("{}", Rand::random_u8().unwrap());
}
