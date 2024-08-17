use position::Position;

mod mmove;
pub mod piece;
mod position;

fn main() {
    let mut pos = Position::default();

    println!("{pos}");

    println!("Moves: {:#?}", pos.movegen());

    pos.make_move(pos.movegen()[1]);

    println!("{pos}");

    println!("Moves: {:#?}", pos.movegen());

    pos.make_move(pos.movegen()[2]);

    println!("{pos}");
    println!("Moves: {:#?}", pos.movegen());
    pos.make_move(pos.movegen()[6]);
    println!("{pos}");
    println!("Moves: {:#?}", pos.movegen());
}
