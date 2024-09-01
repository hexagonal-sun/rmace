use strum::IntoEnumIterator;

use crate::{
    mmove::Move,
    piece::{Colour, Piece, PieceKind},
};

use super::Position;

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
    fn in_check(&self, colour: Colour) -> bool {
        let their_colour = colour.next();
        let king_loc = self[Piece::new(PieceKind::King, colour)]
            .iter_pieces()
            .next()
            .unwrap();

        self.loc_attacked_by_queen(king_loc, their_colour)
            || self.loc_attacked_by_bishop(king_loc, their_colour)
            || self.loc_attacked_by_knight(king_loc, their_colour)
            || self.loc_attacked_by_rook(king_loc, their_colour)
            || self.loc_attacked_by_pawn(king_loc, their_colour)
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

        let mut ret = Vec::new();
        for m in self.movegen() {
            let token = self.make_move(m);
            ret.push((m, _perft(self, depth - 1)));
            self.undo_move(token);
        }

        ret
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
}
