use std::thread;

use strum::IntoEnumIterator;

use crate::{
    mmove::Move,
    piece::{Colour, Piece, PieceKind},
};

use super::{locus::Locus, Position};

mod bishop;
mod king;
mod knight;
mod pawn;
mod queen;
mod rays;
mod rook;

#[cfg(test)]
mod test;

impl Position {
    fn is_loc_under_attack(&self, l: Locus, c: Colour) -> bool {
        self.loc_attacked_by_queen(l, c)
            || self.loc_attacked_by_bishop(l, c)
            || self.loc_attacked_by_knight(l, c)
            || self.loc_attacked_by_rook(l, c)
            || self.loc_attacked_by_pawn(l, c)
            || self.loc_attacked_by_king(l, c)
    }

    fn in_check(&self, colour: Colour) -> bool {
        let their_colour = colour.next();
        let king_loc = self[Piece::new(PieceKind::King, colour)]
            .iter_pieces()
            .next()
            .unwrap();

        self.is_loc_under_attack(king_loc, their_colour)
    }

    pub fn movegen(&mut self) -> Vec<Move> {
        let mut psedo_moves = Vec::new();
        let colour = self.to_play;

        for kind in PieceKind::iter() {
            let piece = Piece::new(kind, self.to_play);

            for src in self[piece].iter_pieces() {
                psedo_moves.append(&mut match kind {
                    PieceKind::Pawn => self.calc_pawn_moves(src),
                    PieceKind::Bishop => self.calc_bishop_moves(src),
                    PieceKind::Knight => self.calc_knight_moves(src),
                    PieceKind::Queen => self.calc_queen_moves(src),
                    PieceKind::Rook => self.calc_rook_moves(src),
                    PieceKind::King => self.calc_king_moves(src),
                });
            }
        }

        psedo_moves.retain(|mmove| {
            let token = self.make_move(*mmove);
            let ret = !self.in_check(colour);
            self.undo_move(token);
            ret
        });

        psedo_moves
    }

    pub fn perft(&mut self, depth: u32) -> Vec<(Move, u32)> {
        fn _perft(pos: &mut Position, depth: u32) -> u32 {
            if depth == 0 {
                return 1;
            }

            let mut n = 0;

            for m in pos.movegen() {
                let token = pos.make_move(m);
                let moves = _perft(pos, depth - 1);
                n += moves;
                pos.undo_move(token);
            }

            n
        }

        let threads: Vec<_> = self
            .movegen()
            .into_iter()
            .map(|m| {
                let mut pos = self.clone();
                pos.make_move(m).consume();
                (m, thread::spawn(move || _perft(&mut pos, depth - 1)))
            })
            .collect();

        let results: Vec<_> = threads
            .into_iter()
            .map(|(m, join)| (m, join.join().unwrap()))
            .collect();

        results
    }
}

#[cfg(test)]
mod tests {
    use crate::{piece::Colour, position::Position};

    #[test]
    fn in_check() {
        let check_pos = [
            "4k3/8/8/4r3/8/8/8/4K3 w - - 0 1",
            "4k3/8/8/8/8/8/8/4K1r1 w - - 0 1",
            "4k3/8/8/8/8/8/8/3qK3 w - - 0 1",
            "4k3/8/8/8/8/2b5/8/4K2b w - - 0 1",
            "4k3/8/8/8/8/3n4/8/4K1n1 w - - 0 1",
            "4k3/8/8/8/8/8/3p4/4K3 w - - 0 1",
        ];

        for pos in check_pos.iter() {
            let pos = Position::from_fen(pos).unwrap();

            assert!(pos.in_check(Colour::White));
        }

        let check_pos = [
            "4K3/8/8/4R3/8/8/8/4k3 w - - 0 1",
            "4K3/8/8/8/8/8/8/4k1R1 w - - 0 1",
            "4K3/8/8/8/8/8/8/3Qk3 w - - 0 1",
            "4K3/8/8/8/8/2B5/8/4k2B w - - 0 1",
            "4K3/8/8/8/8/3N4/8/4k1N1 w - - 0 1",
            "4K3/8/8/8/2k5/3P4/8/8 w - - 0 1",
        ];

        for pos in check_pos.iter() {
            let pos = Position::from_fen(pos).unwrap();

            assert!(pos.in_check(Colour::Black));
        }

        // For sanity, the starting position shouldn't be in check.
        assert!(!Position::default().in_check(Colour::White));
        assert!(!Position::default().in_check(Colour::Black));
    }

    #[test]
    fn perft_starting_pos() {
        let perft_res = Position::default()
            .perft(4)
            .iter()
            .fold(0, |accum, (_, x)| accum + x);

        assert_eq!(perft_res, 197281);
    }

    #[test]
    fn perft_pos1() {
        let perft_res = Position::from_fen(
            "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1",
        )
        .unwrap()
        .perft(4)
        .iter()
        .fold(0, |accum, (_, x)| accum + x);

        assert_eq!(perft_res, 4085603);
    }

    #[test]
    fn perft_pos2() {
        let perft_res = Position::from_fen("8/2p5/3p4/KP5r/1R3p1k/8/4P1P1/8 w - - ")
            .unwrap()
            .perft(5)
            .iter()
            .fold(0, |accum, (_, x)| accum + x);

        assert_eq!(perft_res, 674624);
    }

    #[test]
    fn perft_pos3() {
        let perft_res =
            Position::from_fen("r3k2r/Pppp1ppp/1b3nbN/nP6/BBP1P3/q4N2/Pp1P2PP/R2Q1RK1 w kq - 0 1")
                .unwrap()
                .perft(4)
                .iter()
                .fold(0, |accum, (_, x)| accum + x);

        assert_eq!(perft_res, 422333);
    }

    #[test]
    fn perft_pos4() {
        let perft_res =
            Position::from_fen("rnbq1k1r/pp1Pbppp/2p5/8/2B5/8/PPP1NnPP/RNBQK2R w KQ - 1 8")
                .unwrap()
                .perft(3)
                .iter()
                .fold(0, |accum, (_, x)| accum + x);

        assert_eq!(perft_res, 62379);
    }

    #[test]
    fn perft_pos5() {
        let perft_res = Position::from_fen(
            "r4rk1/1pp1qppp/p1np1n2/2b1p1B1/2B1P1b1/P1NP1N2/1PP1QPPP/R4RK1 w - - 0 10 ",
        )
        .unwrap()
        .perft(4)
        .iter()
        .fold(0, |accum, (_, x)| accum + x);

        assert_eq!(perft_res, 3894594);
    }

    #[test]
    fn perft_pos6() {
        let perft_res = Position::from_fen("8/4r3/4kp2/5b2/r1K2B1P/8/8/8 w - - 3 42")
            .unwrap()
            .perft(3)
            .iter()
            .fold(0, |accum, (_, x)| accum + x);

        assert_eq!(perft_res, 1714);
    }
}
