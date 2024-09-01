use std::{
    fmt::Display,
    io::{self, BufRead},
    time::Instant,
};

use anyhow::{Context, Result};
use clap::Parser;
use nom::{
    bytes::complete::tag,
    character::complete::{digit1, one_of},
    combinator::{map, map_res},
    sequence::tuple,
    Finish, IResult,
};
use rmace::{
    mmove::Move,
    position::{
        locus::{File, Locus, Rank},
        Position,
    },
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

#[derive(PartialEq)]
struct BasicMove {
    src: Locus,
    dst: Locus,
}

impl Display for BasicMove {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.src)?;
        write!(f, "{}", self.dst)
    }
}

impl From<Move> for BasicMove {
    fn from(value: Move) -> Self {
        BasicMove {
            src: value.src,
            dst: value.dst,
        }
    }
}

fn debug(mut moves_made: Vec<BasicMove>, mut pos: Position, depth: u32) -> Result<()> {
    print!("Moves Made: ");
    moves_made.iter().for_each(|m| print!("{} ", m));
    println!("");
    println!("Depth: {}", depth);

    let other_moves = read_other_perft_output()?;

    let perft: Vec<_> = pos
        .perft(depth)
        .iter()
        .map(|(m, x)| (*m, BasicMove::from(*m), *x))
        .collect();

    for other_move in other_moves.iter() {
        if perft
            .iter()
            .filter(|x| other_move.0 == x.1)
            .next()
            .is_none()
        {
            println!("Move {} exists in other engine, but not us.", other_move.0);
            return Ok(());
        }
    }

    for our_moves in perft {
        if let Some(x) = other_moves.iter().filter(|x| our_moves.1 == x.0).next() {
            if x.1 != our_moves.2 {
                println!(
                    "Move {} contains differing perft result. Making move.",
                    our_moves.1
                );
                pos.make_move(our_moves.0).consume();
                moves_made.push(our_moves.1);
                return debug(moves_made, pos.clone(), depth - 1);
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

    let mut position =
        Position::from_fen(args.fen).context("Could not create position from FEN string")?;

    let now = Instant::now();
    let perft = position.perft(args.depth);
    let time_taken = now.elapsed();

    let perft: Vec<_> = perft
        .iter()
        .map(|(m, x)| (BasicMove::from(*m), x))
        .collect();

    for (m, n) in perft.iter() {
        println!("{}: {}", m, n);
    }

    println!("===========");
    println!(
        "Total nodes: {}",
        perft.iter().fold(0, |accum, (_, n)| accum + *n)
    );
    println!("Time taken: {:?}", time_taken);

    if args.debug {
        debug(Vec::new(), position, args.depth)?;
    }

    Ok(())
}

fn parse_rank(input: &str) -> IResult<&str, Rank> {
    map_res(one_of("12345678"), |x| -> Result<Rank, anyhow::Error> {
        let value: u32 = x.to_string().parse()?;
        Ok(Rank::try_from(value)?)
    })(input)
}

fn parse_file(input: &str) -> IResult<&str, File> {
    map(one_of("abcdefgh"), |x| -> File {
        match x {
            'a' => File::A,
            'b' => File::B,
            'c' => File::C,
            'd' => File::D,
            'e' => File::E,
            'f' => File::F,
            'g' => File::G,
            'h' => File::H,
            _ => unreachable!("Parser will only accept valid files"),
        }
    })(input)
}

fn parse_locus(input: &str) -> IResult<&str, Locus> {
    map(tuple((parse_file, parse_rank)), |(f, r)| {
        Locus::from_rank_file(r, f)
    })(input)
}

fn parse_move(input: &str) -> IResult<&str, BasicMove> {
    map(tuple((parse_locus, parse_locus)), |(src, dst)| BasicMove {
        src,
        dst,
    })(input)
}

fn parse_perft_line(input: &str) -> Result<(BasicMove, u32)> {
    Ok(map_res(
        tuple((parse_move, tag(": "), digit1)),
        |(m, _, n)| -> Result<(BasicMove, u32)> { Ok((m, n.parse()?)) },
    )(input)
    .map_err(|e| e.to_owned())
    .finish()
    .map(|x| x.1)?)
}

fn read_other_perft_output() -> Result<Vec<(BasicMove, u32)>> {
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
