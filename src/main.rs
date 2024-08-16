use position::{locus::Locus, movegen::knight::ATTACK_KNIGHT, Position};

pub mod piece;
mod position;

fn main() {
    let pos = Position::default();

    println!("{pos}");
}
