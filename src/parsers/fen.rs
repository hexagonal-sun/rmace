use std::num::ParseIntError;

use nom::{
    branch::alt,
    bytes::complete::tag,
    character::complete::one_of,
    combinator::{map, map_res},
    multi::{many1, separated_list1},
    sequence::tuple,
    IResult,
};

use crate::{
    piece::{Colour, Piece, PieceKind},
    position::castling_rights::CastlingRights,
};

#[derive(Debug)]
pub enum FenElement {
    Piece(Piece),
    Space(u8),
}

#[derive(Debug)]
pub struct Fen {
    pub board: Vec<Vec<FenElement>>,
    pub colour: Colour,
    pub castling_rights: CastlingRights,
}

fn parse_space(input: &str) -> IResult<&str, FenElement> {
    map_res(
        one_of("12345678"),
        |x| -> Result<FenElement, ParseIntError> { Ok(FenElement::Space(x.to_string().parse()?)) },
    )(input)
}

fn parse_piece(input: &str) -> IResult<&str, FenElement> {
    map(one_of("rnbkqpRNBKQP"), |x| {
        let kind = match x.to_ascii_lowercase() {
            'r' => PieceKind::Rook,
            'n' => PieceKind::Knight,
            'b' => PieceKind::Bishop,
            'q' => PieceKind::Queen,
            'k' => PieceKind::King,
            'p' => PieceKind::Pawn,
            _ => unreachable!("other chars not allowed to be parsed"),
        };

        FenElement::Piece(Piece::new(
            kind,
            if x.is_ascii_uppercase() {
                Colour::White
            } else {
                Colour::Black
            },
        ))
    })(input)
}

fn parse_element(input: &str) -> IResult<&str, FenElement> {
    alt((parse_piece, parse_space))(input)
}

fn parse_rank(input: &str) -> IResult<&str, Vec<FenElement>> {
    many1(parse_element)(input)
}

fn parse_board(input: &str) -> IResult<&str, Vec<Vec<FenElement>>> {
    separated_list1(tag("/"), parse_rank)(input)
}

fn parse_colour(input: &str) -> IResult<&str, Colour> {
    map(one_of("wb"), |x| match x {
        'w' => Colour::White,
        'b' => Colour::Black,
        _ => unreachable!("should only parse 'w' or 'b'"),
    })(input)
}

fn parse_castling_rights(input: &str) -> IResult<&str, CastlingRights> {
    let mut rights = CastlingRights::empty();

    alt((
        map(tag("-"), move |_| rights),
        map(many1(one_of("kqKQ")), move |x| {
            x.iter().for_each(|r| match *r {
                'k' => rights[Colour::Black].set_king_side(),
                'q' => rights[Colour::Black].set_queen_side(),
                'K' => rights[Colour::White].set_king_side(),
                'Q' => rights[Colour::White].set_queen_side(),
                _ => unreachable!("Should only parse 'kqKQ'"),
            });

            rights
        }),
    ))(input)
}

pub fn parse_fen(input: &str) -> IResult<&str, Fen> {
    map(
        tuple((
            parse_board,
            tag(" "),
            parse_colour,
            tag(" "),
            parse_castling_rights,
        )),
        |(b, _, c, _, cr)| Fen {
            board: b,
            colour: c,
            castling_rights: cr,
        },
    )(input)
}
