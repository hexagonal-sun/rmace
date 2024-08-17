use crate::{
    mmove::{Move, PieceMove},
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

pub fn calc_knight_moves(position: &Position) -> Vec<Move> {
    let mut ret = Vec::new();
    let piece = Piece::new(PieceKind::Knight, position.to_play);
    let bb = position[piece];
    let blockers = position.blockers();

    for src in bb.iter_pieces() {
        let moves = KNIGHT_MOVES[src.to_idx() as usize];

        for (op, obb) in position.iter_opponent_bbds() {
            for dst in (moves & obb).iter_pieces() {
                ret.push(Move::Attack(PieceMove { piece, src, dst }, op))
            }
        }

        for dst in (moves & !(blockers & moves)).iter_pieces() {
            ret.push(Move::Move(PieceMove { piece, src, dst }))
        }
    }

    ret
}

#[cfg(test)]
mod tests {
    use crate::{
        mmove::{Move, PieceMove},
        piece::mkp,
        position::{bitboard::BitBoard, locus::loc, Position},
    };

    #[test]
    fn simple() {
        let mut p = Position::empty();
        let src = loc!(D, Four);
        let piece = mkp!(White, Knight);
        p[piece] = p[piece].set_piece_at(src);
        let moves = p.movegen();

        assert_eq!(moves.len(), 8);
        for l in [
            loc!(E, Six),
            loc!(F, Five),
            loc!(F, Three),
            loc!(E, Two),
            loc!(C, Two),
            loc!(B, Three),
            loc!(B, Five),
            loc!(C, Six),
        ] {
            assert!(moves.contains(&Move::Move(PieceMove { piece, src, dst: l })));
        }
    }

    #[test]
    fn attacks() {
        let mut p = Position::empty();
        let src = loc!(D, Four);
        let piece = mkp!(White, Knight);
        p[piece] = p[piece].set_piece_at(src);
        p[mkp!(Black, Pawn)] = BitBoard::empty()
            .set_piece_at(loc!(C, Two))
            .set_piece_at(loc!(F, Three));
        let moves = p.movegen();

        assert_eq!(moves.len(), 8);
        for l in [
            loc!(E, Six),
            loc!(F, Five),
            loc!(E, Two),
            loc!(B, Three),
            loc!(B, Five),
            loc!(C, Six),
        ] {
            assert!(moves.contains(&Move::Move(PieceMove { piece, src, dst: l })));
        }

        for l in [loc!(C, Two), loc!(F, Three)] {
            assert!(moves.contains(&Move::Attack(
                PieceMove { piece, src, dst: l },
                mkp!(Black, Pawn)
            )));
        }
    }
}
