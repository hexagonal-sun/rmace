use crate::{
    mmove::{HasDst, MoveBuilder},
    piece::{Colour, Piece, PieceKind},
    position::{
        bitboard::BitBoard,
        locus::{Locus, Rank},
    },
};

use super::{MoveGen, MoveList};

#[derive(Clone, Copy)]
struct PawnMove {
    bb: BitBoard,
    promotes: bool,
}

impl PawnMove {
    const fn empty() -> Self {
        Self {
            bb: BitBoard::empty(),
            promotes: false,
        }
    }
}

const W_PAWN_MOVES: [PawnMove; 64] = calc_w_pawn_moves();
const B_PAWN_MOVES: [PawnMove; 64] = calc_b_pawn_moves();
const W_PAWN_ATTACKS: [PawnMove; 64] = calc_w_pawn_attacks();
const B_PAWN_ATTACKS: [PawnMove; 64] = calc_b_pawn_attacks();

const PROMOTION_KINDS: [PieceKind; 4] = [
    PieceKind::Bishop,
    PieceKind::Knight,
    PieceKind::Queen,
    PieceKind::Rook,
];

macro_rules! unwrap {
    ($e:expr $(,)*) => {
        match $e {
            ::core::option::Option::Some(x) => x,
            ::core::option::Option::None => panic!("Unarap a none"),
        }
    };
}

const fn calc_w_pawn_attacks() -> [PawnMove; 64] {
    let mut table: [PawnMove; 64] = [PawnMove::empty(); 64];
    let mut idx = 0;

    while idx < 64 {
        table[idx] = calc_pawn_attack(unwrap!(Locus::from_idx(idx as u8)), Colour::White);
        idx += 1;
    }

    table
}

const fn calc_b_pawn_attacks() -> [PawnMove; 64] {
    let mut table: [PawnMove; 64] = [PawnMove::empty(); 64];
    let mut idx = 0;

    while idx < 64 {
        table[idx] = calc_pawn_attack(unwrap!(Locus::from_idx(idx as u8)), Colour::Black);
        idx += 1;
    }

    table
}

const fn calc_w_pawn_moves() -> [PawnMove; 64] {
    let mut table: [PawnMove; 64] = [PawnMove::empty(); 64];
    let mut idx = 0;

    while idx < 64 {
        table[idx] = calc_pawn_move(unwrap!(Locus::from_idx(idx as u8)), Colour::White);
        idx += 1;
    }

    table
}

const fn calc_b_pawn_moves() -> [PawnMove; 64] {
    let mut table: [PawnMove; 64] = [PawnMove::empty(); 64];
    let mut idx = 0;

    while idx < 64 {
        table[idx] = calc_pawn_move(unwrap!(Locus::from_idx(idx as u8)), Colour::Black);
        idx += 1;
    }

    table
}

const fn calc_pawn_move(l: Locus, c: Colour) -> PawnMove {
    let bb = BitBoard::empty();
    let is_white = c as u8 == Colour::White as u8;
    let home_rank = if is_white { Rank::Two } else { Rank::Seven };
    let (src_r, _) = l.to_rank_file();

    if src_r as u8 == Rank::One as u8 || src_r as u8 == Rank::Eight as u8 {
        return PawnMove {
            bb: BitBoard::empty(),
            promotes: false,
        };
    }

    let mv = unwrap!(if is_white { l.north() } else { l.south() });
    let home_mv = if src_r as u8 == home_rank as u8 {
        unwrap!(if is_white { mv.north() } else { mv.south() })
    } else {
        mv
    };
    let (dst_r, _) = mv.to_rank_file();

    let bb = bb.set_piece_at(mv).set_piece_at(home_mv);

    let promotion_rank = if is_white { Rank::Eight } else { Rank::One };

    PawnMove {
        bb,
        promotes: dst_r as u8 == promotion_rank as u8,
    }
}

const fn calc_pawn_attack(l: Locus, c: Colour) -> PawnMove {
    let mut bb = BitBoard::empty();
    let is_white = c as u8 == Colour::White as u8;

    let m = if is_white { l.north() } else { l.south() };
    let m = if m.is_none() {
        return PawnMove {
            bb: BitBoard::empty(),
            promotes: false,
        };
    } else {
        unwrap!(m)
    };
    let mv_first = m.east();
    let mv_second = m.west();
    let (dst_r, _) = m.to_rank_file();

    if mv_first.is_some() {
        bb = bb.set_piece_at(unwrap!(mv_first));
    }

    if mv_second.is_some() {
        bb = bb.set_piece_at(unwrap!(mv_second));
    }

    let promotion_rank = if is_white { Rank::Eight } else { Rank::One };

    PawnMove {
        bb,
        promotes: dst_r as u8 == promotion_rank as u8,
    }
}

impl MoveGen<'_> {
    fn add_pawn_promotions(ml: &mut MoveList, builder: MoveBuilder<HasDst>, c: Colour) {
        for kind in PROMOTION_KINDS {
            ml.push(builder.with_pawn_promotion(Piece::new(kind, c)).build())
        }
    }

    pub fn calc_pawn_moves(&mut self, src: Locus) {
        let piece = Piece::new(PieceKind::Pawn, self.position.to_play);
        let blockers = self.blockers;
        let mgen = MoveBuilder::new(piece, src);

        let (moves, attacks) = if self.position.to_play == Colour::White {
            (
                W_PAWN_MOVES[src.to_idx() as usize],
                W_PAWN_ATTACKS[src.to_idx() as usize],
            )
        } else {
            (
                B_PAWN_MOVES[src.to_idx() as usize],
                B_PAWN_ATTACKS[src.to_idx() as usize],
            )
        };

        for (op, obb) in self.position.iter_opponent_bbds() {
            for dst in (attacks.bb & obb).iter_pieces() {
                let b = mgen.with_dst(dst).with_capture(op);
                if attacks.promotes {
                    Self::add_pawn_promotions(&mut self.moves, b, self.position.to_play);
                } else {
                    self.moves.push(b.build());
                }
            }
        }

        let home_blocker_mask = if self.position.to_play == Colour::White {
            BitBoard::new(0xff0000)
        } else {
            BitBoard::new(0xff0000000000)
        };

        if let Some(ep_loc) = self.position.en_passant {
            if attacks.bb.has_piece_at(ep_loc) {
                self.moves.push(mgen.with_dst(ep_loc).build())
            }
        }

        if !(moves.bb & blockers & home_blocker_mask).is_empty() {
            return;
        }

        for dst in (moves.bb & !(blockers & moves.bb)).iter_pieces() {
            let b = mgen.with_dst(dst);

            let (src_rank, _) = src.to_rank_file();
            let (dst_rank, _) = dst.to_rank_file();

            if (src_rank == Rank::Two && dst_rank == Rank::Four)
                || (src_rank == Rank::Seven && dst_rank == Rank::Five)
            {
                self.moves.push(b.sets_ep().build());
                continue;
            }

            if moves.promotes {
                Self::add_pawn_promotions(&mut self.moves, b, self.position.to_play);
            } else {
                self.moves.push(b.build());
            }
        }
    }

    pub fn loc_attacked_by_pawn(&self, l: Locus, c: Colour) -> bool {
        let attacks = if c == Colour::White {
            B_PAWN_ATTACKS[l.to_idx() as usize].bb
        } else {
            W_PAWN_ATTACKS[l.to_idx() as usize].bb
        };

        !(self.position[Piece::new(PieceKind::Pawn, c)] & attacks).is_empty()
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        mmove::MoveBuilder,
        piece::{mkp, Colour, Piece},
        position::{
            builder::PositionBuilder,
            locus::{loc, Locus},
            movegen::{pawn::PROMOTION_KINDS, MoveGen},
            Position,
        },
    };

    macro_rules! pmoves {
        ($pos:expr, $src:expr) => {{
            let mut mgen = MoveGen::new($pos);
            mgen.calc_pawn_moves($src);
            mgen.moves
        }};
    }

    #[test]
    fn loc_attack_white() {
        let mut pos = PositionBuilder::new()
            .with_piece_at(mkp!(White, Pawn), loc!(c 4))
            .with_piece_at(mkp!(White, Pawn), loc!(e 6))
            .with_piece_at(mkp!(White, Pawn), loc!(g 7))
            .build();

        let mgen = MoveGen::new(&mut pos);

        for loc in Locus::iter_all_squares() {
            if loc == loc!(b 5)
                || loc == loc!(d 5)
                || loc == loc!(d 7)
                || loc == loc!(f 7)
                || loc == loc!(f 8)
                || loc == loc!(h 8)
            {
                assert!(mgen.loc_attacked_by_pawn(loc, Colour::White))
            } else {
                assert!(!mgen.loc_attacked_by_pawn(loc, Colour::White))
            }
        }
    }

    #[test]
    fn loc_attack_black() {
        let mut pos = PositionBuilder::new()
            .with_piece_at(mkp!(Black, Pawn), loc!(c 4))
            .build();

        let mgen = MoveGen::new(&mut pos);

        for loc in Locus::iter_all_squares() {
            if loc == loc!(b 3) || loc == loc!(d 3) {
                assert!(mgen.loc_attacked_by_pawn(loc, Colour::Black))
            } else {
                assert!(!mgen.loc_attacked_by_pawn(loc, Colour::Black))
            }
        }
    }

    #[test]
    fn home_rank_moves() {
        let src = loc!(b 2);
        let piece = mkp!(White, Pawn);
        let mut p = PositionBuilder::new()
            .with_piece_at(piece, src)
            .with_next_turn(Colour::White)
            .build();
        let moves = pmoves!(&mut p, src);
        let mgen = MoveBuilder::new(piece, src);

        assert_eq!(moves.len(), 2);
        assert!(moves.contains(&mgen.with_dst(loc!(b 3)).build()));
        assert!(moves.contains(&mgen.with_dst(loc!(b 4)).sets_ep().build()));

        let src = loc!(d 7);
        let piece = mkp!(Black, Pawn);
        let mut p = PositionBuilder::new()
            .with_piece_at(piece, src)
            .with_next_turn(Colour::Black)
            .build();

        let moves = pmoves!(&mut p, src);
        let mgen = MoveBuilder::new(piece, src);

        assert_eq!(moves.len(), 2);
        assert!(moves.contains(&mgen.with_dst(loc!(d 6)).build()));
        assert!(moves.contains(&mgen.with_dst(loc!(d 5)).sets_ep().build()));
    }

    #[test]
    fn home_rank_blocks() {
        let mut p = PositionBuilder::new()
            .with_piece_at(mkp!(White, Pawn), loc!(b 2))
            .with_piece_at(mkp!(Black, Knight), loc!(b 3))
            .with_piece_at(mkp!(White, Pawn), loc!(e 2))
            .with_piece_at(mkp!(White, Queen), loc!(e 3))
            .with_next_turn(Colour::White)
            .build();
        assert!(pmoves!(&mut p, loc!(b 2)).is_empty());
        assert!(pmoves!(&mut p, loc!(e 2)).is_empty());

        let mut p = PositionBuilder::new()
            .with_piece_at(mkp!(Black, Pawn), loc!(b 7))
            .with_piece_at(mkp!(White, Knight), loc!(b 6))
            .with_piece_at(mkp!(Black, Pawn), loc!(e 7))
            .with_piece_at(mkp!(Black, Queen), loc!(e 6))
            .with_next_turn(Colour::Black)
            .build();
        assert!(pmoves!(&mut p, loc!(b 7)).is_empty());
        assert!(pmoves!(&mut p, loc!(e 7)).is_empty());
    }

    #[test]
    fn opponent_blocks() {
        let mut p = PositionBuilder::new()
            .with_piece_at(mkp!(White, Pawn), loc!(b 4))
            .with_piece_at(mkp!(Black, Knight), loc!(b 5))
            .with_next_turn(Colour::White)
            .build();

        assert_eq!(pmoves!(&mut p, loc!(b 4)).len(), 0);
    }

    #[test]
    fn standard_moves() {
        let src = loc!(b 4);
        let piece = mkp!(White, Pawn);
        let mut p = PositionBuilder::new()
            .with_piece_at(piece, src)
            .with_next_turn(Colour::White)
            .build();
        let moves = pmoves!(&mut p, src);

        assert_eq!(moves.len(), 1);
        assert!(moves.contains(&MoveBuilder::new(piece, src).with_dst(loc!(b 5)).build()));

        let src = loc!(d 5);
        let piece = mkp!(Black, Pawn);
        let mut p = PositionBuilder::new()
            .with_piece_at(piece, src)
            .with_next_turn(Colour::Black)
            .build();
        let moves = pmoves!(&mut p, src);

        assert_eq!(moves.len(), 1);
        assert!(moves.contains(&MoveBuilder::new(piece, src).with_dst(loc!(d 4)).build()));
    }

    #[test]
    fn attacks() {
        let src = loc!(b 4);
        let piece = mkp!(White, Pawn);
        let mut p = PositionBuilder::new()
            .with_piece_at(piece, src)
            .with_piece_at(mkp!(Black, Pawn), loc!(a 5))
            .with_piece_at(mkp!(Black, Pawn), loc!(c 5))
            .with_next_turn(Colour::White)
            .build();
        let moves = pmoves!(&mut p, src);
        let mgen = MoveBuilder::new(piece, src);

        assert_eq!(moves.len(), 3);
        assert!(moves.contains(&mgen.with_dst(loc!(b 5)).build()));

        for l in [loc!(a 5), loc!(c 5)] {
            assert!(moves.contains(&mgen.with_dst(l).with_capture(mkp!(Black, Pawn)).build()));
        }

        let src = loc!(d 5);
        let piece = mkp!(Black, Pawn);
        let mut p = PositionBuilder::new()
            .with_piece_at(piece, src)
            .with_piece_at(mkp!(White, Pawn), loc!(c 4))
            .with_piece_at(mkp!(White, Pawn), loc!(e 4))
            .with_next_turn(Colour::Black)
            .build();
        let moves = pmoves!(&mut p, src);
        let mgen = MoveBuilder::new(piece, src);

        assert_eq!(moves.len(), 3);
        assert!(moves.contains(&mgen.with_dst(loc!(d 4)).build()));
        for l in [loc!(c 4), loc!(e 4)] {
            assert!(moves.contains(&mgen.with_dst(l).with_capture(mkp!(White, Pawn)).build()));
        }
    }

    #[test]
    fn en_passant_capture() {
        let mut pos =
            Position::from_fen("rnbqkb1r/pppppppp/5n2/P7/8/8/8/RNBQKBNR b KQkq - 0 2").unwrap();

        pos.make_move(
            MoveBuilder::new(mkp!(Black, Pawn), loc!(b 7))
                .with_dst(loc!(b 5))
                .sets_ep()
                .build(),
        )
        .consume();
        assert_eq!(pmoves!(&mut pos, loc!(a 5)).len(), 2);
    }

    #[test]
    fn promotions() {
        let src = loc!(b 7);
        let piece = mkp!(White, Pawn);
        let mut p = PositionBuilder::new()
            .with_piece_at(piece, src)
            .with_next_turn(Colour::White)
            .build();
        let moves = pmoves!(&mut p, src);

        assert_eq!(moves.len(), PROMOTION_KINDS.len());
        for k in PROMOTION_KINDS {
            assert!(moves.contains(
                &MoveBuilder::new(piece, src)
                    .with_dst(loc!(b 8))
                    .with_pawn_promotion(Piece::new(k, Colour::White))
                    .build()
            ));
        }

        let src = loc!(d 2);
        let piece = mkp!(Black, Pawn);
        let mut p = PositionBuilder::new()
            .with_piece_at(piece, src)
            .with_next_turn(Colour::Black)
            .build();
        let moves = pmoves!(&mut p, src);

        assert_eq!(moves.len(), PROMOTION_KINDS.len());
        for k in PROMOTION_KINDS {
            assert!(moves.contains(
                &MoveBuilder::new(piece, src)
                    .with_dst(loc!(d 1))
                    .with_pawn_promotion(Piece::new(k, Colour::Black))
                    .build()
            ));
        }
    }
}
