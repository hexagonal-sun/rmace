use crate::piece::{Colour, Piece, PieceKind};
use rand::{RngCore, SeedableRng};
use rand_chacha::ChaCha8Rng;
use strum::IntoEnumIterator;

use super::{
    locus::{loc, File, Locus},
    Position,
};

pub type ZobristKey = u64;

#[derive(Clone, PartialEq)]
pub struct Zobrist {
    piece_sq_tables: [[ZobristKey; 64]; 12],
    btm: ZobristKey,
    castling_rights: [ZobristKey; 4],
    ep_file: [ZobristKey; 8],
}

impl Zobrist {
    pub fn new() -> Self {
        let mut rng = ChaCha8Rng::seed_from_u64(0xdeadbeefd00dfeed);
        let mut piece_sq_tables = [[0; 64]; 12];
        let btm = rng.next_u64();
        let mut castling_rights = [0; 4];
        let mut ep_file = [0; 8];

        for t in piece_sq_tables.iter_mut() {
            for x in t.iter_mut() {
                *x = rng.next_u64();
            }
        }

        for x in castling_rights.iter_mut() {
            *x = rng.next_u64();
        }

        for x in ep_file.iter_mut() {
            *x = rng.next_u64();
        }

        Self {
            piece_sq_tables,
            btm,
            castling_rights,
            ep_file,
        }
    }

    pub fn piece_loc_key(&self, p: Piece, loc: Locus) -> ZobristKey {
        self.piece_sq_tables[p.to_idx()][loc.to_idx() as usize]
    }

    pub fn btm_key(&self) -> ZobristKey {
        self.btm
    }

    pub fn castling_rights_key(&self, c: Colour, loc: Locus) -> ZobristKey {
        let (_, f) = loc.to_rank_file();

        let f_idx = match f {
            File::A => 0,
            File::H => 1,
            _ => return 0,
        };

        self.castling_rights[c as usize * 2 + f_idx]
    }

    pub fn ep_key(&self, loc: Locus) -> ZobristKey {
        let (_, f) = loc.to_rank_file();

        self.ep_file[f as usize]
    }

    pub fn from_position(&self, pos: &Position) -> ZobristKey {
        let mut key = 0;

        for p in PieceKind::iter() {
            for c in Colour::iter() {
                let p = Piece::new(p, c);
                for pos in pos[p].iter_pieces() {
                    key ^= self.piece_loc_key(p, pos);
                }
            }
        }

        if pos.to_play() == Colour::Black {
            key ^= self.btm_key();
        }

        for c in Colour::iter() {
            let crights = pos.castling_rights[c];
            if crights.queen_side() {
                key ^= self.castling_rights_key(c, loc!(a 1));
            }

            if crights.king_side() {
                key ^= self.castling_rights_key(c, loc!(h 1));
            }
        }

        if let Some(ep) = pos.en_passant {
            key ^= self.ep_key(ep);
        }

        key
    }
}

#[cfg(test)]
mod tests {
    use crate::position::Position;

    #[test]
    fn start_not_zero() {
        let pos = Position::default();

        assert_ne!(pos.hash, 0);
    }
}
