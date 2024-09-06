use std::fmt::Display;

use nom::{
    character::complete::one_of,
    combinator::{map, map_res, opt},
    sequence::tuple,
    IResult,
};

use crate::{
    mmove::Move,
    piece::PieceKind,
    position::locus::{File, Locus, Rank},
};

#[derive(PartialEq, Debug)]
pub struct UciMove {
    pub src: Locus,
    pub dst: Locus,
    pub promote: Option<PieceKind>,
}

fn parse_rank(input: &str) -> IResult<&str, Rank> {
    map_res(one_of("12345678"), |x| -> Result<Rank, anyhow::Error> {
        let value: u32 = x.to_string().parse()?;
        Rank::try_from(value)
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

fn parse_promotion(input: &str) -> IResult<&str, PieceKind> {
    map(one_of("qrnb"), |x| match x {
        'q' => PieceKind::Queen,
        'r' => PieceKind::Rook,
        'n' => PieceKind::Knight,
        'b' => PieceKind::Bishop,
        _ => unreachable!("Should only parse 'qrnb'"),
    })(input)
}

pub fn parse_uci_move(input: &str) -> IResult<&str, UciMove> {
    map(
        tuple((parse_locus, parse_locus, opt(parse_promotion))),
        |(src, dst, promote)| UciMove { src, dst, promote },
    )(input)
}

impl Display for UciMove {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.src)?;
        write!(f, "{}", self.dst)?;

        if let Some(p) = self.promote {
            write!(f, "{}", p)?;
        }

        Ok(())
    }
}

impl From<Move> for UciMove {
    fn from(value: Move) -> Self {
        UciMove {
            src: value.src,
            dst: value.dst,
            promote: value.promote.map(|x| x.kind()),
        }
    }
}
