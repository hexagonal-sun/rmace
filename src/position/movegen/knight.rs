use crate::{
    mmove::MoveBuilder,
    piece::{Colour, Piece, PieceKind},
    position::{bitboard::BitBoard, locus::Locus},
};

use super::MoveGen;

const KNIGHT_MOVES: [BitBoard; 64] = calc_attack_knight();

macro_rules! gen_knight_attack {
    ($func_name:ident, $m1:ident, $m2:ident, $m3:ident) => {
        const fn $func_name(locus: Locus) -> Option<BitBoard> {
            match locus.$m1() {
                Some(l) => match l.$m2() {
                    Some(l) => match l.$m3() {
                        Some(l) => return Some(l.to_bitboard()),
                        None => return None,
                    },
                    None => return None,
                },
                None => return None,
            }
        }
    };
}

gen_knight_attack!(gen_knight_attack_nnw, north, north, west);
gen_knight_attack!(gen_knight_attack_nne, north, north, east);
gen_knight_attack!(gen_knight_attack_een, east, east, north);
gen_knight_attack!(gen_knight_attack_ees, east, east, south);
gen_knight_attack!(gen_knight_attack_sse, south, south, east);
gen_knight_attack!(gen_knight_attack_ssw, south, south, west);
gen_knight_attack!(gen_knight_attack_wws, west, west, south);
gen_knight_attack!(gen_knight_attack_wwn, west, west, north);

const fn gen_knight_attacks(locus: Locus) -> BitBoard {
    BitBoard::empty()
        .opt_or(gen_knight_attack_nnw(locus))
        .opt_or(gen_knight_attack_nne(locus))
        .opt_or(gen_knight_attack_een(locus))
        .opt_or(gen_knight_attack_ees(locus))
        .opt_or(gen_knight_attack_sse(locus))
        .opt_or(gen_knight_attack_ssw(locus))
        .opt_or(gen_knight_attack_wws(locus))
        .opt_or(gen_knight_attack_wwn(locus))
}

const fn calc_attack_knight() -> [BitBoard; 64] {
    let mut table: [BitBoard; 64] = [BitBoard::empty(); 64];
    let mut idx = 0;

    while idx < 64 {
        match Locus::from_idx(idx as u8) {
            Some(l) => table[idx] = gen_knight_attacks(l),
            None => unreachable!(),
        }
        idx += 1;
    }

    table
}

impl MoveGen<'_> {
    pub fn calc_knight_moves(&mut self, src: Locus) {
        let piece = Piece::new(PieceKind::Knight, self.position.to_play);
        let mgen = MoveBuilder::new(piece, src);
        let moves = KNIGHT_MOVES[src.to_idx() as usize];

        for (op, obb) in self.position.iter_opponent_bbds() {
            for dst in (moves & obb).iter_pieces() {
                self.moves.push(mgen.with_dst(dst).with_capture(op).build())
            }
        }

        for dst in (moves & !(self.blockers & moves)).iter_pieces() {
            self.moves.push(mgen.with_dst(dst).build())
        }
    }

    pub fn loc_attacked_by_knight(&self, l: Locus, c: Colour) -> bool {
        !(self.position[Piece::new(PieceKind::Knight, c)] & KNIGHT_MOVES[l.to_idx() as usize])
            .is_empty()
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        mmove::MoveBuilder,
        piece::{mkp, Colour},
        position::{
            builder::PositionBuilder,
            locus::{loc, Locus},
            movegen::{test::mk_test, MoveGen},
        },
    };

    #[test]
    fn loc_attack() {
        let mut pos = PositionBuilder::new()
            .with_piece_at(mkp!(White, Knight), loc!(c 4))
            .build();

        let attacked_squares = [
            loc!(d 6),
            loc!(e 5),
            loc!(e 3),
            loc!(d 2),
            loc!(b 2),
            loc!(a 3),
            loc!(a 5),
            loc!(b 6),
        ];

        let mut mgen = MoveGen::new(&mut pos);

        for loc in Locus::iter_all_squares() {
            if attacked_squares.contains(&loc) {
                assert!(mgen.loc_attacked_by_knight(loc, Colour::White));
            } else {
                assert!(!mgen.loc_attacked_by_knight(loc, Colour::White));
            }
        }
    }

    mk_test!(name=simple,
             calc_fn=calc_knight_moves,
             kind=Knight,
             src=loc!(d 4),
             blockers=;
             attacks=;
             moves=loc!(e 6),
                   loc!(f 5),
                   loc!(f 3),
                   loc!(e 2),
                   loc!(c 2),
                   loc!(b 3),
                   loc!(b 5),
                   loc!(c 6));

    mk_test!(name=blockers,
             calc_fn=calc_knight_moves,
             kind=Knight,
             src=loc!(d 4),
             blockers=loc!(c 2), loc!(f 3);
             attacks=;
             moves=loc!(e 6),
                   loc!(f 5),
                   loc!(e 2),
                   loc!(b 3),
                   loc!(b 5),
                   loc!(c 6));

    mk_test!(name=attacks,
             calc_fn=calc_knight_moves,
             kind=Knight,
             src=loc!(d 4),
             blockers=loc!(c 2), loc!(f 3);
             attacks=loc!(f 5), loc!(b 3), loc!(b 5);
             moves=loc!(e 6),
                   loc!(e 2),
                   loc!(c 6));
}
