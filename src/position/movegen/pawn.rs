use crate::{
    mmove::{HasDst, Move, MoveBuilder},
    piece::{Colour, Piece, PieceKind},
    position::{
        bitboard::BitBoard,
        locus::{Locus, Rank},
        Position,
    },
};

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
    let (src_r, _) = l.to_rank_file();

    if src_r as u8 == Rank::One as u8 || src_r as u8 == Rank::Eight as u8 {
        return PawnMove {
            bb: BitBoard::empty(),
            promotes: false,
        };
    }

    let m = unwrap!(if is_white { l.north() } else { l.south() });
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

fn add_promotions(moves: &mut Vec<Move>, builder: MoveBuilder<HasDst>, colour: Colour) {
    for kind in PROMOTION_KINDS {
        moves.push(
            builder
                .with_pawn_promotion(Piece::new(kind, colour))
                .build(),
        )
    }
}

impl Position {
    pub fn calc_pawn_moves(&self) -> Vec<Move> {
        let mut ret = Vec::new();
        let piece = Piece::new(PieceKind::Pawn, self.to_play);

        let bb = self[piece];
        let blockers = self.blockers();

        for src in bb.iter_pieces() {
            let mgen = MoveBuilder::new(piece, src);

            let (moves, attacks) = if self.to_play == Colour::White {
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

            for (op, obb) in self.iter_opponent_bbds() {
                for dst in (attacks.bb & obb).iter_pieces() {
                    let b = mgen.with_dst(dst).with_capture(op);
                    if attacks.promotes {
                        add_promotions(&mut ret, b, self.to_play);
                    } else {
                        ret.push(b.build());
                    }
                }
            }

            for dst in (moves.bb & !(blockers & moves.bb)).iter_pieces() {
                let b = mgen.with_dst(dst);

                if moves.promotes {
                    add_promotions(&mut ret, b, self.to_play);
                } else {
                    ret.push(b.build());
                }
            }
        }

        ret
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        mmove::MoveBuilder,
        piece::{mkp, Colour, Piece},
        position::{builder::PositionBuilder, locus::loc, movegen::pawn::PROMOTION_KINDS},
    };

    #[test]
    fn home_rank_moves() {
        let src = loc!(b 2);
        let piece = mkp!(White, Pawn);
        let p = PositionBuilder::new()
            .with_piece_at(piece, src)
            .with_next_turn(Colour::White)
            .build();
        let moves = p.calc_pawn_moves();
        let mgen = MoveBuilder::new(piece, src);

        assert_eq!(moves.len(), 2);

        for l in [loc!(b 3), loc!(b 4)] {
            assert!(moves.contains(&mgen.with_dst(l).build()));
        }

        let src = loc!(d 7);
        let piece = mkp!(Black, Pawn);
        let p = PositionBuilder::new()
            .with_piece_at(piece, src)
            .with_next_turn(Colour::Black)
            .build();
        let moves = p.calc_pawn_moves();
        let mgen = MoveBuilder::new(piece, src);

        assert_eq!(moves.len(), 2);

        for l in [loc!(d 6), loc!(d 5)] {
            assert!(moves.contains(&mgen.with_dst(l).build()));
        }
    }

    #[test]
    fn opponent_blocks() {
        let p = PositionBuilder::new()
            .with_piece_at(mkp!(White, Pawn), loc!(b 4))
            .with_piece_at(mkp!(Black, Knight), loc!(b 5))
            .with_next_turn(Colour::White)
            .build();
        let moves = p.calc_pawn_moves();

        assert_eq!(moves.len(), 0);
    }

    #[test]
    fn standard_moves() {
        let src = loc!(b 4);
        let piece = mkp!(White, Pawn);
        let p = PositionBuilder::new()
            .with_piece_at(piece, src)
            .with_next_turn(Colour::White)
            .build();
        let moves = p.calc_pawn_moves();

        assert_eq!(moves.len(), 1);
        assert!(moves.contains(&MoveBuilder::new(piece, src).with_dst(loc!(b 5)).build()));

        let src = loc!(d 5);
        let piece = mkp!(Black, Pawn);
        let p = PositionBuilder::new()
            .with_piece_at(piece, src)
            .with_next_turn(Colour::Black)
            .build();
        let moves = p.calc_pawn_moves();

        assert_eq!(moves.len(), 1);
        assert!(moves.contains(&MoveBuilder::new(piece, src).with_dst(loc!(d 4)).build()));
    }

    #[test]
    fn attacks() {
        let src = loc!(b 4);
        let piece = mkp!(White, Pawn);
        let p = PositionBuilder::new()
            .with_piece_at(piece, src)
            .with_piece_at(mkp!(Black, Pawn), loc!(a 5))
            .with_piece_at(mkp!(Black, Pawn), loc!(c 5))
            .with_next_turn(Colour::White)
            .build();
        let moves = p.calc_pawn_moves();
        let mgen = MoveBuilder::new(piece, src);

        assert_eq!(moves.len(), 3);
        assert!(moves.contains(&mgen.with_dst(loc!(b 5)).build()));

        for l in [loc!(a 5), loc!(c 5)] {
            assert!(moves.contains(&mgen.with_dst(l).with_capture(mkp!(Black, Pawn)).build()));
        }

        let src = loc!(d 5);
        let piece = mkp!(Black, Pawn);
        let p = PositionBuilder::new()
            .with_piece_at(piece, src)
            .with_piece_at(mkp!(White, Pawn), loc!(c 4))
            .with_piece_at(mkp!(White, Pawn), loc!(e 4))
            .with_next_turn(Colour::Black)
            .build();
        let moves = p.calc_pawn_moves();
        let mgen = MoveBuilder::new(piece, src);

        assert_eq!(moves.len(), 3);
        assert!(moves.contains(&mgen.with_dst(loc!(d 4)).build()));
        for l in [loc!(c 4), loc!(e 4)] {
            assert!(moves.contains(&mgen.with_dst(l).with_capture(mkp!(White, Pawn)).build()));
        }
    }

    #[test]
    fn promotions() {
        let src = loc!(b 7);
        let piece = mkp!(White, Pawn);
        let p = PositionBuilder::new()
            .with_piece_at(piece, src)
            .with_next_turn(Colour::White)
            .build();
        let moves = p.calc_pawn_moves();

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
        let p = PositionBuilder::new()
            .with_piece_at(piece, src)
            .with_next_turn(Colour::Black)
            .build();
        let moves = p.calc_pawn_moves();

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
