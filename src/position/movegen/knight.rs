use crate::{
    mmove::{Move, MoveBuilder},
    piece::{Piece, PieceKind},
    position::{bitboard::BitBoard, locus::Locus, Position},
};

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

impl Position {
    pub fn calc_knight_moves(&self) -> Vec<Move> {
        let mut ret = Vec::new();
        let piece = Piece::new(PieceKind::Knight, self.to_play);
        let bb = self[piece];
        let blockers = self.blockers();

        for src in bb.iter_pieces() {
            let mgen = MoveBuilder::new(piece, src);
            let moves = KNIGHT_MOVES[src.to_idx() as usize];

            for (op, obb) in self.iter_opponent_bbds() {
                for dst in (moves & obb).iter_pieces() {
                    ret.push(mgen.with_dst(dst).with_capture(op).build())
                }
            }

            for dst in (moves & !(blockers & moves)).iter_pieces() {
                ret.push(mgen.with_dst(dst).build())
            }
        }

        ret
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        mmove::MoveBuilder,
        piece::mkp,
        position::{builder::PositionBuilder, locus::loc},
    };

    #[test]
    fn simple() {
        let src = loc!(d 4);
        let piece = mkp!(White, Knight);
        let p = PositionBuilder::new().with_piece_at(piece, src).build();
        let moves = p.calc_knight_moves();
        let mgen = MoveBuilder::new(piece, src);

        assert_eq!(moves.len(), 8);
        for l in [
            loc!(e 6),
            loc!(f 5),
            loc!(f 3),
            loc!(e 2),
            loc!(c 2),
            loc!(b 3),
            loc!(b 5),
            loc!(c 6),
        ] {
            assert!(moves.contains(&mgen.with_dst(l).build()));
        }
    }

    #[test]
    fn blockers() {
        let src = loc!(d 4);
        let piece = mkp!(White, Knight);
        let p = PositionBuilder::new()
            .with_piece_at(piece, src)
            .with_piece_at(mkp!(White, Pawn), loc!(c 2))
            .with_piece_at(mkp!(White, Pawn), loc!(f 3))
            .build();
        let moves = p.calc_knight_moves();
        let mgen = MoveBuilder::new(piece, src);

        assert_eq!(moves.len(), 6);
        for l in [
            loc!(e 6),
            loc!(f 5),
            loc!(e 2),
            loc!(b 3),
            loc!(b 5),
            loc!(c 6),
        ] {
            assert!(moves.contains(&mgen.with_dst(l).build()))
        }
    }

    #[test]
    fn attacks() {
        let src = loc!(d 4);
        let piece = mkp!(White, Knight);
        let p = PositionBuilder::new()
            .with_piece_at(piece, src)
            .with_piece_at(mkp!(Black, Pawn), loc!(c 2))
            .with_piece_at(mkp!(Black, Pawn), loc!(f 3))
            .build();
        let moves = p.calc_knight_moves();
        let mgen = MoveBuilder::new(piece, src);

        assert_eq!(moves.len(), 8);
        for l in [
            loc!(e 6),
            loc!(f 5),
            loc!(e 2),
            loc!(b 3),
            loc!(b 5),
            loc!(c 6),
        ] {
            assert!(moves.contains(&mgen.with_dst(l).build()));
        }

        for l in [loc!(c 2), loc!(f 3)] {
            assert!(moves.contains(&mgen.with_dst(l).with_capture(mkp!(Black, Pawn)).build()));
        }
    }
}
