use std::{
    fmt::Display,
    ops::{Index, IndexMut},
};

use bitboard::BitBoard;
use locus::{loc, File, Locus, Rank};
use strum::{EnumCount, IntoEnumIterator};

use crate::{
    mmove::Move,
    piece::{mkp, Colour, Piece, PieceKind, PieceKindIter},
};

pub mod bitboard;
pub mod builder;
pub mod locus;
pub mod movegen;

#[derive(Clone, PartialEq)]
pub struct Position {
    bboards: [BitBoard; PieceKind::COUNT * 2],
    to_play: Colour,
}

impl Position {
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

    pub fn blockers(&self) -> BitBoard {
        let mut b = BitBoard::empty();

        for bb in self.bboards.iter() {
            b = b.or(*bb);
        }

        b
    }

    pub fn movegen(&self) -> Vec<Move> {
        let mut ret = Vec::new();

        ret.append(&mut self.calc_knight_moves());
        ret.append(&mut self.calc_pawn_moves());

        ret
    }

    pub fn make_move(&mut self, mmove: Move) {
        match mmove {
            Move::Move {
                piece,
                src,
                dst,
                capture,
                promote,
            } => {
                self[piece] = self[piece].clear_piece_at(src).set_piece_at(dst);

                if let Some(fallen) = capture {
                    self[fallen] = self[fallen].clear_piece_at(dst);
                }

                if let Some(promotion) = promote {
                    self[piece] = self[piece].clear_piece_at(dst);
                    self[promotion] = self[promotion].set_piece_at(dst);
                }
            }
        }

        self.to_play = self.to_play.next();
    }

    pub fn empty() -> Self {
        Self {
            bboards: [BitBoard::empty(); PieceKind::COUNT * 2],
            to_play: Colour::White,
        }
    }

    pub fn iter_opponent_bbds(&self) -> OpponentBbIter {
        OpponentBbIter {
            pos: &self,
            p: PieceKind::iter(),
        }
    }
}

pub struct OpponentBbIter<'a> {
    pos: &'a Position,
    p: PieceKindIter,
}

impl Iterator for OpponentBbIter<'_> {
    type Item = (Piece, BitBoard);

    fn next(&mut self) -> Option<Self::Item> {
        let kind = self.p.next()?;
        let piece = Piece::new(kind, self.pos.to_play.next());

        Some((piece, self.pos[piece]))
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

impl Index<Piece> for Position {
    type Output = BitBoard;

    fn index(&self, index: Piece) -> &Self::Output {
        self.bboards.index(index.idx as usize)
    }
}

impl IndexMut<Piece> for Position {
    fn index_mut(&mut self, index: Piece) -> &mut Self::Output {
        self.bboards.index_mut(index.idx as usize)
    }
}

impl Default for Position {
    fn default() -> Self {
        let mut pos = Self::empty();

        let pawns = 0b11111111;
        let rooks = 0b10000001;
        let knights = 0b01000010;
        let bishops = 0b00100100;

        pos[mkp!(White, Pawn)] = BitBoard::new(pawns << 8);
        pos[mkp!(Black, Pawn)] = BitBoard::new(pawns << 48);
        pos[mkp!(White, Rook)] = BitBoard::new(rooks);
        pos[mkp!(Black, Rook)] = BitBoard::new(rooks << 56);
        pos[mkp!(White, Knight)] = BitBoard::new(knights);
        pos[mkp!(Black, Knight)] = BitBoard::new(knights << 56);
        pos[mkp!(White, Bishop)] = BitBoard::new(bishops);
        pos[mkp!(Black, Bishop)] = BitBoard::new(bishops << 56);
        pos[mkp!(White, King)] = loc!(E, One).to_bitboard();
        pos[mkp!(Black, King)] = loc!(E, Eight).to_bitboard();
        pos[mkp!(White, Queen)] = loc!(D, One).to_bitboard();
        pos[mkp!(Black, Queen)] = loc!(D, Eight).to_bitboard();

        pos
    }
}
