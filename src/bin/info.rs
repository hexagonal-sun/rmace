use std::io::{stdin, stdout, BufRead, Write};

use anyhow::Result;
use rmace::position::Position;

fn main() -> Result<()> {
    print!("Enter FEN String: ");
    stdout().flush()?;

    let mut fen_string = String::new();
    stdin().lock().read_line(&mut fen_string)?;

    let pos = Position::from_fen(fen_string)?;

    println!("{}", pos);

    Ok(())
}
