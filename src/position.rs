use std::fmt::Display;

use bitboard::BitBoard;
use locus::{loc, File, Locus, Rank};
use strum::{EnumCount, IntoEnumIterator};

use crate::piece::{Colour, Piece, PieceKind};

pub mod bitboard;
pub mod locus;
pub mod movegen;

#[derive(Clone, PartialEq)]
pub struct Position {
    bboards: [BitBoard; PieceKind::COUNT * 2],
    to_play: Colour,
}

macro_rules! pidx {
    ($colour:ident, $kind:ident) => {
        crate::piece::PieceKind::$kind as usize + crate::piece::Colour::$colour as usize
    };
}

impl Position {
    pub fn lookup(&self, colour: Colour, kind: PieceKind) -> BitBoard {
        self.bboards[colour as usize + kind as usize]
    }

    pub fn piece_at_loc(&self, l: Locus) -> Option<Piece> {
        for p in PieceKind::iter() {
            let w_bb = self.bboards[p as usize];
            let b_bb = self.bboards[p as usize + PieceKind::COUNT];

            if w_bb.has_piece_at(l) {
                return Some(Piece::new(p, Colour::White));
            }

            if b_bb.has_piece_at(l) {
                return Some(Piece::new(p, Colour::Black));
            }
        }

        None
    }

    pub fn movegen(&self) -> Vec<Position> {
        todo!()
    }
}

impl Display for Position {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for rank in Rank::iter().rev() {
            write!(f, "{rank:?} ")?;
            for file in File::iter() {
                write!(
                    f,
                    "{}",
                    self.piece_at_loc(Locus::from_rank_file(rank, file))
                        .map(|x| format!("{x} "))
                        .unwrap_or(". ".to_string())
                )?;
            }
            write!(f, "\n")?;
        }

        write!(f, "  ")?;
        for file in File::iter() {
            write!(f, "{file:?} ")?;
        }

        Ok(())
    }
}

impl Default for Position {
    fn default() -> Self {
        let pawns   = 0b11111111;
        let rooks   = 0b10000001;
        let knights = 0b01000010;
        let bishops = 0b00100100;

        let mut bboards: [BitBoard; PieceKind::COUNT * 2] = [BitBoard::empty(); PieceKind::COUNT * 2];
        bboards[pidx!(White, Pawn)] = BitBoard::new(pawns << 8);
        bboards[pidx!(Black, Pawn)] = BitBoard::new(pawns << 48);
        bboards[pidx!(White, Rook)] = BitBoard::new(rooks);
        bboards[pidx!(Black, Rook)] = BitBoard::new(rooks << 56);
        bboards[pidx!(White, Knight)] = BitBoard::new(knights);
        bboards[pidx!(Black, Knight)] = BitBoard::new(knights << 56);
        bboards[pidx!(White, Bishop)] = BitBoard::new(bishops);
        bboards[pidx!(Black, Bishop)] = BitBoard::new(bishops << 56);
        bboards[pidx!(White, King)] = loc!(E, One).to_bitboard();
        bboards[pidx!(Black, King)] = loc!(E, Eight).to_bitboard();
        bboards[pidx!(White, Queen)] = loc!(D, One).to_bitboard();
        bboards[pidx!(Black, Queen)] = loc!(D, Eight).to_bitboard();

        Self {
            bboards,
            to_play: Colour::White,
        }
    }
}
