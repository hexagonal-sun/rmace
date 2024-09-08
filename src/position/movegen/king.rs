use paste::paste;

use crate::{
    mmove::{CastlingMoveType, Move, MoveBuilder},
    piece::{Colour, Piece, PieceKind},
    position::{
        bitboard::BitBoard,
        locus::{loc, File, Locus},
        Position,
    },
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

const BLOCKER_MASK_KINGSIDE: [BitBoard; 2] = [
    BitBoard::empty()
        .set_piece_at(loc!(f 1))
        .set_piece_at(loc!(g 1)),
    BitBoard::empty()
        .set_piece_at(loc!(f 8))
        .set_piece_at(loc!(g 8)),
];

const BLOCKER_MASK_QUEENSIDE: [BitBoard; 2] = [
    BitBoard::empty()
        .set_piece_at(loc!(b 1))
        .set_piece_at(loc!(c 1))
        .set_piece_at(loc!(d 1)),
    BitBoard::empty()
        .set_piece_at(loc!(b 8))
        .set_piece_at(loc!(c 8))
        .set_piece_at(loc!(d 8)),
];

const CHECK_SQ_KINGSIDE: [Locus; 2] = [loc!(f 1), loc!(f 8)];
const CHECK_SQ_QUEENSIDE: [Locus; 2] = [loc!(d 1), loc!(d 8)];

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
        let (r, _) = src.to_rank_file();

        for (op, obb) in self.iter_opponent_bbds() {
            for dst in (moves & obb).iter_pieces() {
                ret.push(mgen.with_dst(dst).with_capture(op).build())
            }
        }

        for dst in (moves & !(blockers & moves)).iter_pieces() {
            ret.push(mgen.with_dst(dst).build())
        }

        let castling_rights = self.castling_rights[self.to_play];
        if castling_rights.king_side()
            && (BLOCKER_MASK_KINGSIDE[self.to_play as usize] & blockers).is_empty()
            && !self.is_loc_under_attack(
                CHECK_SQ_KINGSIDE[self.to_play as usize],
                self.to_play.next(),
            )
            && !self.in_check(self.to_play)
        {
            ret.push(
                mgen.with_dst(Locus::from_rank_file(r, File::G))
                    .is_castling_move(CastlingMoveType::Kingside)
                    .build(),
            );
        }

        if castling_rights.queen_side()
            && (BLOCKER_MASK_QUEENSIDE[self.to_play as usize] & blockers).is_empty()
            && !self.is_loc_under_attack(
                CHECK_SQ_QUEENSIDE[self.to_play as usize],
                self.to_play.next(),
            )
            && !self.in_check(self.to_play)
        {
            ret.push(
                mgen.with_dst(Locus::from_rank_file(r, File::C))
                    .is_castling_move(CastlingMoveType::Queenside)
                    .build(),
            );
        }

        ret
    }

    pub fn loc_attacked_by_king(&self, l: Locus, c: Colour) -> bool {
        !(self[Piece::new(PieceKind::King, c)] & KING_MOVES[l.to_idx() as usize]).is_empty()
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        mmove::MoveBuilder,
        piece::mkp,
        position::{builder::PositionBuilder, locus::loc, movegen::test::mk_test, Position},
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

    #[test]
    fn castle_queen_side() {
        let pos = Position::from_fen(
            "rnbqkbnr/p4ppp/1p1p4/2p1p3/5P2/1QPPB3/PP1NP1PP/R3KBNR w KQkq - 0 7",
        )
        .unwrap();
        assert_eq!(dbg!(pos.calc_king_moves(loc!(e 1))).len(), 3);

        let pos =
            Position::from_fen("rnbqkbnr/p4ppp/1p1p4/2p1p3/5P2/1QPPB3/PP1NP1PP/R3KBNR w - - 0 7")
                .unwrap();
        assert_eq!(dbg!(pos.calc_king_moves(loc!(e 1))).len(), 2);

        let pos =
            Position::from_fen("r3kbnr/p4ppp/n7/1p1ppP2/qP1P4/N2Q4/P2B1PPP/R3KBNR w KQkq - 1 13")
                .unwrap();
        assert_eq!(dbg!(pos.calc_king_moves(loc!(e 1))).len(), 2);

        let pos = Position::from_fen("r3kbnr/8/8/8/8/8/PPPPPPPP/RNBQKBNR b KQkq - 0 1").unwrap();
        assert_eq!(dbg!(pos.calc_king_moves(loc!(e 8))).len(), 5);

        let pos = Position::from_fen("r3kbnr/8/8/8/8/8/PPPPPPPP/RNBQKBNR b - - 0 1").unwrap();
        assert_eq!(dbg!(pos.calc_king_moves(loc!(e 8))).len(), 4);

        let pos = Position::from_fen("r3kbnr/8/8/6Q1/8/8/PPPPPPPP/RNB1KBNR b KQkq - 0 1").unwrap();
        assert_eq!(dbg!(pos.calc_king_moves(loc!(e 8))).len(), 4);
    }

    #[test]
    fn castle_king_side() {
        // Gen white castling move.
        let pos =
            Position::from_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQK2R w KQkq - 0 1").unwrap();
        assert_eq!(pos.calc_king_moves(loc!(e 1)).len(), 2);

        // Don't gen if no castling rights.
        assert_eq!(
            Position::from_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQK2R w - - 0 1")
                .unwrap()
                .calc_king_moves(loc!(e 1))
                .len(),
            1
        );

        // Don't gen if blockers.
        assert_eq!(
            Position::from_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQK1NR w KQkq - 0 1")
                .unwrap()
                .calc_king_moves(loc!(e 1))
                .len(),
            1
        );

        // Don't gen if sq under attack
        assert_eq!(
            Position::from_fen("rn1qkbnr/pppppppp/8/1b6/8/8/PPPP2PP/RNBQK2R w KQkq - 0 1")
                .unwrap()
                .calc_king_moves(loc!(e 1))
                .len(),
            3
        );
    }

    #[test]
    fn no_castling_moves_in_check() {
        let pos = Position::from_fen(
            "r3k2r/p1p1qpb1/bn1ppnp1/1B1PN3/1p2P3/P1N2Q1p/1PPB1PPP/R3K2R b KQkq - 1 2",
        )
        .unwrap();

        assert_eq!(pos.calc_king_moves(loc!(e 8)).len(), 3);
    }
}
