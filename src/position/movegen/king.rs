use paste::paste;

use crate::{
    mmove::{Move, MoveBuilder},
    piece::{Piece, PieceKind},
    position::{bitboard::BitBoard, locus::Locus, Position},
};

const KING_MOVES: [BitBoard; 64] = gen_king_moves();

macro_rules! gen_king_move {
    ($dir:ident) => {
        paste! {
           const fn [<gen_king_move_ $dir>](locus: Locus) -> Option<BitBoard> {
               match locus.$dir() {
                   Some(l) => return Some(l.to_bitboard()),
                   None => return None,
               }
           }
        }
    };
    ($dir1:ident, $dir2:ident) => {
        paste! {
           const fn [<gen_king_move_ $dir1 _ $dir2>](locus: Locus) -> Option<BitBoard> {
               match locus.$dir1() {
                   Some(l) => match l.$dir2() {
                       Some(l) => return Some(l.to_bitboard()),
                       None => return None,
                   }
                   None => return None,
               }
           }
        }
    };
}

gen_king_move!(north);
gen_king_move!(east);
gen_king_move!(south);
gen_king_move!(west);
gen_king_move!(north, east);
gen_king_move!(north, west);
gen_king_move!(south, west);
gen_king_move!(south, east);

const fn gen_king_move(locus: Locus) -> BitBoard {
    BitBoard::empty()
        .opt_or(gen_king_move_north(locus))
        .opt_or(gen_king_move_east(locus))
        .opt_or(gen_king_move_south(locus))
        .opt_or(gen_king_move_west(locus))
        .opt_or(gen_king_move_north_east(locus))
        .opt_or(gen_king_move_north_west(locus))
        .opt_or(gen_king_move_south_east(locus))
        .opt_or(gen_king_move_south_west(locus))
}

const fn gen_king_moves() -> [BitBoard; 64] {
    let mut table: [BitBoard; 64] = [BitBoard::empty(); 64];
    let mut idx = 0;

    while idx < 64 {
        match Locus::from_idx(idx as u8) {
            Some(l) => table[idx] = gen_king_move(l),
            None => unreachable!(),
        }
        idx += 1;
    }

    table
}

impl Position {
    pub fn calc_king_moves(&self, src: Locus) -> Vec<Move> {
        let mut ret = Vec::new();
        let piece = Piece::new(PieceKind::King, self.to_play);
        let blockers = self.blockers();
        let mgen = MoveBuilder::new(piece, src);
        let moves = KING_MOVES[src.to_idx() as usize];

        for (op, obb) in self.iter_opponent_bbds() {
            for dst in (moves & obb).iter_pieces() {
                ret.push(mgen.with_dst(dst).with_capture(op).build())
            }
        }

        for dst in (moves & !(blockers & moves)).iter_pieces() {
            ret.push(mgen.with_dst(dst).build())
        }

        ret
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        mmove::MoveBuilder,
        piece::mkp,
        position::{builder::PositionBuilder, locus::loc, movegen::test::mk_test},
    };

    mk_test!(name=simple,
             calc_fn=calc_king_moves,
             kind=King,
             src=loc!(d 4),
             blockers=;
             attacks=;
             moves=loc!(d 5),
                   loc!(e 5),
                   loc!(c 5),
                   loc!(d 3),
                   loc!(e 3),
                   loc!(c 3),
                   loc!(e 4),
                   loc!(c 4));

    mk_test!(name=blockers,
             calc_fn=calc_king_moves,
             kind=King,
             src=loc!(d 4),
             blockers=loc!(c 5), loc!(d 5), loc!(c 4);
             attacks=;
             moves=loc!(e 5),
                   loc!(d 3),
                   loc!(e 3),
                   loc!(c 3),
                   loc!(e 4));

    mk_test!(name=attacks,
             calc_fn=calc_king_moves,
             kind=King,
             src=loc!(d 4),
             blockers=loc!(c 5), loc!(d 5), loc!(c 4);
             attacks=loc!(c 3), loc!(e 3), loc!(d 3);
             moves=loc!(e 5),
                   loc!(e 4));
}
