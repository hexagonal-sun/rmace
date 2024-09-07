use std::io::{self, BufRead};

use anyhow::{Context, Result};
use nom::{
    branch::alt,
    bytes::complete::tag,
    character::complete::multispace0,
    combinator::{map, opt},
    error::ParseError,
    multi::many1,
    sequence::{delimited, tuple},
    Finish, IResult, Parser,
};
use rmace::{
    parsers::{
        fen::{parse_fen, Fen},
        uci_move::{parse_uci_move, UciMove},
    },
    position::Position,
    search::SearchBuilder,
};

#[derive(Debug)]
enum PosSpecifier {
    Starpos,
    Fen(Fen),
}

#[derive(Debug)]
enum UciCmd {
    Uci,
    IsReady,
    NewGame,
    Position(PosSpecifier, Option<Vec<UciMove>>),
    Go,
    Display,
}

fn parse_cmd_uci(input: &str) -> IResult<&str, UciCmd> {
    map(tag("uci"), |_| UciCmd::Uci)(input)
}

fn parse_cmd_isready(input: &str) -> IResult<&str, UciCmd> {
    map(tag("isready"), |_| UciCmd::IsReady)(input)
}

fn parse_cmd_newgame(input: &str) -> IResult<&str, UciCmd> {
    map(tag("ucinewgame"), |_| UciCmd::NewGame)(input)
}

fn ws<'a, O, E: ParseError<&'a str>, F>(inner: F) -> impl Parser<&'a str, O, E>
where
    F: Parser<&'a str, O, E>,
{
    delimited(multispace0, inner, multispace0)
}

fn parse_cmd_position(input: &str) -> IResult<&str, UciCmd> {
    map(
        tuple((
            ws(tag("position")),
            alt((
                map(tuple((ws(tag("fen")), parse_fen)), |(_, fen)| {
                    PosSpecifier::Fen(fen)
                }),
                map(ws(tag("startpos")), |_| PosSpecifier::Starpos),
            )),
            opt(tuple((
                ws(tag("moves")),
                many1(delimited(multispace0, parse_uci_move, multispace0)),
            ))),
        )),
        |(_, pos, moves)| UciCmd::Position(pos, moves.map(|x| x.1)),
    )(input)
}

fn parse_cmd_go(input: &str) -> IResult<&str, UciCmd> {
    map(tag("go"), |_| UciCmd::Go)(input)
}

fn parse_uci_cmd(input: &str) -> Result<UciCmd> {
    Ok(alt((
        parse_cmd_uci,
        parse_cmd_isready,
        parse_cmd_newgame,
        parse_cmd_position,
        parse_cmd_go,
        map(tag("d"), |_| UciCmd::Display),
    ))(input)
    .map_err(|e| e.to_owned())
    .finish()
    .map(|x| x.1)?)
}

fn main() -> Result<()> {
    let mut pos = Position::default();
    loop {
        let mut line = String::new();
        io::stdin()
            .lock()
            .read_line(&mut line)
            .context("Failed to read UCI line")?;

        let cmd = parse_uci_cmd(&line)?;
        match cmd {
            UciCmd::Uci => handle_cmd_uci(),
            UciCmd::IsReady => handle_cmd_isready(),
            UciCmd::NewGame => handle_cmd_newgame(&mut pos),
            UciCmd::Position(f, m) => handle_cmd_position(&mut pos, f, m),
            UciCmd::Go => handle_cmd_go(&mut pos),
            UciCmd::Display => println!("{}", pos),
        }
    }
}

fn handle_cmd_newgame(pos: &mut Position) {
    *pos = Position::default();
}

fn handle_cmd_go(pos: &mut Position) {
    let m = SearchBuilder::new(pos.clone()).build().go();

    println!("bestmove {}", UciMove::from(m))
}

fn handle_cmd_position(pos: &mut Position, p: PosSpecifier, m: Option<Vec<UciMove>>) {
    match p {
        PosSpecifier::Fen(fen) => {
            *pos = Position::try_from(fen).expect("Could not create position from FEN")
        }
        PosSpecifier::Starpos => *pos = Position::default(),
    }

    if let Some(moves) = m {
        for m in moves.iter() {
            match pos.movegen().iter().find(|x| {
                x.src == m.src && x.dst == m.dst && x.promote.map(|x| x.kind()) == m.promote
            }) {
                Some(x) => pos.make_move(*x).consume(),
                None => panic!("Move {} is not a valid move", m),
            }
        }
    }
}

fn handle_cmd_isready() {
    println!("readyok");
}

fn handle_cmd_uci() {
    println!("id rmace");
    println!("id author Matthew Leach");
    println!("uciok");
}
