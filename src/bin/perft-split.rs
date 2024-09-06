use std::{
    io::{self, BufRead},
    time::Instant,
};

use anyhow::{Context, Result};
use clap::Parser;
use nom::{
    bytes::complete::tag, character::complete::digit1, combinator::map_res, sequence::tuple, Finish,
};
use rmace::{
    parsers::uci_move::{parse_uci_move, UciMove},
    position::Position,
};

#[derive(clap::Parser)]
/// Run perft, a function for debugging rmace's movegen and testing it's
/// performace.
struct Args {
    /// The depth of the perft search.
    depth: u32,

    /// The FEN string of the position to perform perft on.
    fen: String,

    /// Whether to enter debugging mode. This allows you to enter the perft
    /// split output from another chess engine to facilitate easy movegen
    /// debugging.
    #[arg(short, long)]
    debug: bool,
}

fn debug(
    original_pos: String,
    mut moves_made: Vec<UciMove>,
    mut pos: Position,
    depth: u32,
) -> Result<()> {
    print!("position fen \"{}\" moves ", original_pos);
    moves_made.iter().for_each(|m| print!("{} ", m));
    println!();
    println!("Depth: {}", depth);

    let other_moves = read_other_perft_output()?;

    let perft: Vec<_> = pos
        .perft(depth)
        .iter()
        .map(|(m, x)| (*m, UciMove::from(*m), *x))
        .collect();

    for other_move in other_moves.iter() {
        if !perft.iter().any(|x| other_move.0 == x.1) {
            println!("Move {} exists in other engine, but not us.", other_move.0);
            return Ok(());
        }
    }

    for our_moves in perft {
        if let Some(x) = other_moves.iter().find(|x| our_moves.1 == x.0) {
            if x.1 != our_moves.2 {
                println!(
                    "Move {} contains differing perft result. Making move.",
                    our_moves.1
                );
                pos.make_move(our_moves.0).consume();
                moves_made.push(our_moves.1);
                return debug(original_pos, moves_made, pos.clone(), depth - 1);
            }
        } else {
            println!(
                "Move {} exists in our engine, but not the other.",
                our_moves.1
            );
            return Ok(());
        }
    }

    println!("Perft results match.");

    Ok(())
}

fn main() -> Result<()> {
    let args = Args::parse();

    let mut position = Position::from_fen(args.fen.clone())
        .context("Could not create position from FEN string")?;

    let now = Instant::now();
    let perft = position.perft(args.depth);
    let time_taken = now.elapsed();

    let perft: Vec<_> = perft.iter().map(|(m, x)| (UciMove::from(*m), x)).collect();

    for (m, n) in perft.iter() {
        println!("{}: {}", m, n);
    }

    let total_nodes = perft.iter().fold(0, |accum, (_, n)| accum + *n);

    println!("===========");
    println!("Total nodes: {}", total_nodes);
    println!("Time taken: {:?}", time_taken);
    println!(
        "Node per second: {}",
        total_nodes as f32 / time_taken.as_secs_f32()
    );

    if args.debug {
        debug(args.fen, Vec::new(), position, args.depth)?;
    }

    Ok(())
}

fn parse_perft_line(input: &str) -> Result<(UciMove, u32)> {
    Ok(map_res(
        tuple((parse_uci_move, tag(": "), digit1)),
        |(m, _, n)| -> Result<(UciMove, u32)> { Ok((m, n.parse()?)) },
    )(input)
    .map_err(|e| e.to_owned())
    .finish()
    .map(|x| x.1)?)
}

fn read_other_perft_output() -> Result<Vec<(UciMove, u32)>> {
    println!("Enter the perft output from another engine, followed by a blank newline:");
    let stdin = io::stdin();
    let mut ret = Vec::new();

    for line in stdin.lock().lines() {
        let line = line.context("Could not read line")?;

        if line.is_empty() {
            break;
        }

        let perft_result = parse_perft_line(&line).context("Could not parse perft line")?;
        ret.push(perft_result);
    }

    Ok(ret)
}
