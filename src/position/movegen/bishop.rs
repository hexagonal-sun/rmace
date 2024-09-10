use crate::{
    mmove::MoveBuilder,
    piece::{Colour, Piece, PieceKind},
    position::{bitboard::BitBoard, locus::Locus},
};

use super::{
    rays::{
        calc_north_east_rays_moves, calc_north_west_rays_moves, calc_south_east_rays_moves,
        calc_south_west_rays_moves,
    },
    MoveGen,
};

fn rays(src: Locus, blockers: BitBoard) -> BitBoard {
    calc_north_west_rays_moves(src, blockers)
        | calc_north_east_rays_moves(src, blockers)
        | calc_south_west_rays_moves(src, blockers)
        | calc_south_east_rays_moves(src, blockers)
}

impl MoveGen<'_> {
    pub fn calc_bishop_moves(&mut self, src: Locus) {
        let p = Piece::new(PieceKind::Bishop, self.position.to_play);
        let our_pieces = self.position.all_pieces_for_colour(self.position.to_play);
        let their_pieces = self
            .position
            .all_pieces_for_colour(self.position.to_play.next());
        let builder = MoveBuilder::new(p, src);

        for dst in (rays(src, self.blockers) & (!our_pieces)).iter_pieces() {
            let mut m = builder.with_dst(dst);

            if their_pieces.has_piece_at(dst) {
                let piece = self
                    .position
                    .piece_at_loc(dst)
                    .expect("their_pieces bb has piece at loc");

                m = m.with_capture(piece);
            }

            self.moves.push(m.build());
        }
    }

    pub fn loc_attacked_by_bishop(&self, l: Locus, c: Colour) -> bool {
        !(self.position[Piece::new(PieceKind::Bishop, c)] & rays(l, self.blockers)).is_empty()
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
            .with_piece_at(mkp!(White, Bishop), loc!(c 4))
            .with_piece_at(mkp!(White, Pawn), loc!(e 6))
            .with_piece_at(mkp!(Black, Pawn), loc!(d 3))
            .build();

        let attacked_squares = [
            loc!(b 3),
            loc!(a 2),
            loc!(b 5),
            loc!(a 6),
            loc!(d 5),
            loc!(e 6),
            loc!(d 3),
        ];

        let mgen = MoveGen::new(&mut pos);

        for loc in Locus::iter_all_squares() {
            if attacked_squares.contains(&loc) {
                assert!(mgen.loc_attacked_by_bishop(loc, Colour::White));
            } else {
                assert!(!mgen.loc_attacked_by_bishop(loc, Colour::White));
            }
        }
    }

    mk_test!(name=simple,
             calc_fn=calc_bishop_moves,
             kind=Bishop,
             src=loc!(d 4),
             blockers=;
             attacks=;
             moves=loc!(c 5),
                   loc!(b 6),
                   loc!(a 7),
                   loc!(e 5),
                   loc!(f 6),
                   loc!(g 7),
                   loc!(h 8),
                   loc!(e 3),
                   loc!(f 2),
                   loc!(g 1),
                   loc!(c 3),
                   loc!(b 2),
                   loc!(a 1));

    mk_test!(name=blockers,
             calc_fn=calc_bishop_moves,
             kind=Bishop,
             src=loc!(d 4),
             blockers=loc!(b 6), loc!(f 6), loc!(c 3);
             attacks=;
             moves=loc!(c 5),
                   loc!(e 5),
                   loc!(e 3),
                   loc!(f 2),
                   loc!(g 1));

    mk_test!(name=attacks,
             calc_fn=calc_bishop_moves,
             kind=Bishop,
             src=loc!(d 4),
             blockers=loc!(b 6), loc!(f 6);
             attacks=loc!(f 2), loc!(c 3);
             moves=loc!(c 5),
                   loc!(e 5),
                   loc!(e 3));
}
