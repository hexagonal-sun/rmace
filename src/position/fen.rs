use std::{convert::Infallible, fmt::Debug, num::ParseIntError};

use anyhow::{anyhow, bail, Result};
use nom::{
    branch::alt,
    bytes::complete::tag,
    character::complete::one_of,
    combinator::map_res,
    multi::{many1, separated_list1},
    sequence::tuple,
    Finish, IResult,
};
use strum::{EnumCount, IntoEnumIterator};

use crate::piece::{Colour, Piece, PieceKind};

use super::{
    builder::PositionBuilder,
    locus::{file, Locus, Rank},
    Position,
};

#[derive(Debug)]
enum Element {
    Piece(Piece),
    Space(u8),
}

#[derive(Debug)]
struct Fen {
    board: Vec<Vec<Element>>,
    colour: Colour,
}

fn parse_space(input: &str) -> IResult<&str, Element> {
    map_res(one_of("12345678"), |x| -> Result<Element, ParseIntError> {
        Ok(Element::Space(x.to_string().parse()?))
    })(input)
}

fn parse_piece(input: &str) -> IResult<&str, Element> {
    map_res(one_of("rnbkqpRNBKQP"), |x| -> Result<Element, Infallible> {
        let kind = match x.to_ascii_lowercase() {
            'r' => PieceKind::Rook,
            'n' => PieceKind::Knight,
            'b' => PieceKind::Bishop,
            'q' => PieceKind::Queen,
            'k' => PieceKind::King,
            'p' => PieceKind::Pawn,
            _ => unreachable!("other chars not allowed to be parsed"),
        };

        Ok(Element::Piece(Piece::new(
            kind,
            if x.is_ascii_uppercase() {
                Colour::White
            } else {
                Colour::Black
            },
        )))
    })(input)
}

fn parse_element(input: &str) -> IResult<&str, Element> {
    alt((parse_piece, parse_space))(input)
}

fn parse_rank(input: &str) -> IResult<&str, Vec<Element>> {
    many1(parse_element)(input)
}

fn parse_board(input: &str) -> IResult<&str, Vec<Vec<Element>>> {
    separated_list1(tag("/"), parse_rank)(input)
}

fn parse_colour(input: &str) -> IResult<&str, Colour> {
    map_res(one_of("wb"), |x| -> Result<Colour, Infallible> {
        Ok(match x {
            'w' => Colour::White,
            'b' => Colour::Black,
            _ => unreachable!("should only parse 'w' or 'b'"),
        })
    })(input)
}

fn parse_fen(input: &str) -> Result<Fen> {
    map_res(
        tuple((parse_board, tag(" "), parse_colour)),
        |(b, _, c)| -> Result<Fen, Infallible> {
            Ok(Fen {
                board: b,
                colour: c,
            })
        },
    )(input)
    .finish()
    .map_err(|x| anyhow!("Could not parse FEN: {}", x.to_string()))
    .map(|x| x.1)
}

impl Position {
    pub fn from_fen(fen: impl ToString) -> Result<Self> {
        let fen = parse_fen(&fen.to_string())?;

        if fen.board.len() != Rank::COUNT {
            bail!("Invalid number of ranks in FEN string");
        }

        let mut pos = PositionBuilder::new();

        for (elms, rank) in fen.board.iter().zip(Rank::iter().rev()) {
            let mut loc = Some(Locus::from_rank_file(rank, file!(a)));

            fn shift(loc: &mut Option<Locus>) -> Result<()> {
                if let Some(l) = loc {
                    *loc = l.east()
                } else {
                    bail!("Too many pieces in FEN rank specification");
                }

                Ok(())
            }

            for elm in elms.iter() {
                match elm {
                    Element::Piece(p) => {
                        pos = pos.with_piece_at(*p, loc.unwrap());
                        shift(&mut loc)?;
                    }
                    Element::Space(n) => {
                        for _ in 0..*n {
                            shift(&mut loc)?;
                        }
                    }
                }
            }

            if loc.is_some() {
                bail!("Too few pieces/spaces on rank");
            }
        }

        Ok(pos.with_next_turn(fen.colour).build())
    }
}

impl Debug for Position {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Position")
            .field("bboards", &self.bboards)
            .field("to_play", &self.to_play)
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use crate::position::Position;

    #[test]
    fn starting_pos() {
        let result =
            Position::from_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1").unwrap();

        assert_eq!(result, Position::default());
    }

    #[test]
    fn empty() {
        let result = Position::from_fen("8/8/8/8/8/8/8/8 w KQkq - 0 1").unwrap();

        assert_eq!(result, Position::empty());
    }

    #[test]
    #[should_panic]
    fn too_many_pieces_on_rank() {
        Position::from_fen("6bpp/8/8/8/8/8/8/8 w KQkq - 0 1").unwrap();
    }

    #[test]
    #[should_panic]
    fn too_few_pieces_on_rank() {
        Position::from_fen("4bpp/8/8/8/8/8/8/8 w KQkq - 0 1").unwrap();
    }

    #[test]
    #[should_panic]
    fn too_many_ranks() {
        Position::from_fen("7p/8/8/8/8/8/8/8/p7 w KQkq - 0 1").unwrap();
    }

    #[test]
    #[should_panic]
    fn too_few_ranks() {
        Position::from_fen("7p/8/8/8/8/p7 w KQkq - 0 1").unwrap();
    }
}
