use std::fmt::Debug;

use anyhow::{anyhow, bail, Result};
use nom::Finish;
use strum::{EnumCount, IntoEnumIterator};

use crate::{
    parsers::fen::{parse_fen, Fen, FenElement},
    position::locus::file,
};

use super::{
    builder::PositionBuilder,
    locus::{Locus, Rank},
    Position,
};

impl TryFrom<Fen> for Position {
    type Error = anyhow::Error;

    fn try_from(value: Fen) -> std::result::Result<Self, Self::Error> {
        if value.board.len() != Rank::COUNT {
            bail!("Invalid number of ranks in FEN string");
        }

        let mut pos = PositionBuilder::new();

        for (elms, rank) in value.board.iter().zip(Rank::iter().rev()) {
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
                    FenElement::Piece(p) => {
                        pos = pos.with_piece_at(*p, loc.unwrap());
                        shift(&mut loc)?;
                    }
                    FenElement::Space(n) => {
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

        Ok(pos
            .with_castling_rights(value.castling_rights)
            .with_next_turn(value.colour)
            .build())
    }
}

impl Position {
    pub fn from_fen(fen: impl ToString) -> Result<Self> {
        let fen = parse_fen(&fen.to_string())
            .finish()
            .map_err(|x| anyhow!("Could not parse FEN: {}", x.to_string()))
            .map(|x| x.1)?;

        Self::try_from(fen)
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
    use crate::{piece::Colour, position::Position};

    #[test]
    fn starting_pos() {
        let result =
            Position::from_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1").unwrap();

        assert_eq!(result, Position::default());
    }

    #[test]
    fn empty() {
        let result = Position::from_fen("8/8/8/8/8/8/8/8 w - - 0 1").unwrap();

        assert_eq!(result, Position::empty());
    }

    #[test]
    fn castling_rights() {
        let result = Position::from_fen("8/8/8/8/8/8/8/8 w kqK - 0 1").unwrap();

        assert!(result.castling_rights[Colour::White].king_side());
        assert!(!result.castling_rights[Colour::White].queen_side());
        assert!(result.castling_rights[Colour::Black].king_side());
        assert!(result.castling_rights[Colour::Black].queen_side());
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
