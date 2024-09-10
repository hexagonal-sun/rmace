use crate::{
    mmove::MoveBuilder,
    piece::{Colour, Piece, PieceKind},
    position::{bitboard::BitBoard, locus::Locus},
};

use super::{
    rays::{
        calc_east_rays_moves, calc_north_east_rays_moves, calc_north_rays_moves,
        calc_north_west_rays_moves, calc_south_east_rays_moves, calc_south_rays_moves,
        calc_south_west_rays_moves, calc_west_rays_moves,
    },
    MoveGen,
};

fn rays(src: Locus, blockers: BitBoard) -> BitBoard {
    calc_north_west_rays_moves(src, blockers)
        | calc_north_east_rays_moves(src, blockers)
        | calc_south_west_rays_moves(src, blockers)
        | calc_south_east_rays_moves(src, blockers)
        | calc_north_rays_moves(src, blockers)
        | calc_east_rays_moves(src, blockers)
        | calc_south_rays_moves(src, blockers)
        | calc_west_rays_moves(src, blockers)
}

impl MoveGen<'_> {
    pub fn calc_queen_moves(&mut self, src: Locus) {
        let p = Piece::new(PieceKind::Queen, self.position.to_play);
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

    pub fn loc_attacked_by_queen(&self, l: Locus, c: Colour) -> bool {
        !(self.position[Piece::new(PieceKind::Queen, c)] & rays(l, self.blockers)).is_empty()
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
            .with_piece_at(mkp!(White, Queen), loc!(c 4))
            .with_piece_at(mkp!(White, Pawn), loc!(e 6))
            .with_piece_at(mkp!(Black, Pawn), loc!(d 3))
            .with_piece_at(mkp!(White, Pawn), loc!(c 6))
            .with_piece_at(mkp!(Black, Pawn), loc!(f 4))
            .build();

        let attacked_squares = [
            loc!(b 3),
            loc!(a 2),
            loc!(b 5),
            loc!(a 6),
            loc!(d 5),
            loc!(e 6),
            loc!(d 3),
            loc!(c 1),
            loc!(c 2),
            loc!(c 3),
            loc!(c 5),
            loc!(c 6),
            loc!(a 4),
            loc!(b 4),
            loc!(d 4),
            loc!(e 4),
            loc!(f 4),
        ];

        let mgen = MoveGen::new(&mut pos);

        for loc in Locus::iter_all_squares() {
            if attacked_squares.contains(&loc) {
                assert!(mgen.loc_attacked_by_queen(loc, Colour::White));
            } else {
                assert!(!mgen.loc_attacked_by_queen(loc, Colour::White));
            }
        }
    }

    mk_test!(name=simple,
             calc_fn=calc_queen_moves,
             kind=Queen,
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
                   loc!(a 1),
                   loc!(d 1),
                   loc!(d 2),
                   loc!(d 3),
                   loc!(d 5),
                   loc!(d 6),
                   loc!(d 7),
                   loc!(d 8),
                   loc!(a 4),
                   loc!(b 4),
                   loc!(c 4),
                   loc!(e 4),
                   loc!(f 4),
                   loc!(g 4),
                   loc!(h 4));

    mk_test!(name=blockers,
             calc_fn=calc_queen_moves,
             kind=Queen,
             src=loc!(d 4),
             blockers=loc!(f 4), loc!(f 6), loc!(b 2);
             attacks=;
             moves=loc!(c 5),
                   loc!(b 6),
                   loc!(a 7),
                   loc!(e 5),
                   loc!(e 3),
                   loc!(f 2),
                   loc!(g 1),
                   loc!(c 3),
                   loc!(d 1),
                   loc!(d 2),
                   loc!(d 3),
                   loc!(d 5),
                   loc!(d 6),
                   loc!(d 7),
                   loc!(d 8),
                   loc!(a 4),
                   loc!(b 4),
                   loc!(c 4),
                   loc!(e 4));

    mk_test!(name=attacks,
             calc_fn=calc_queen_moves,
             kind=Queen,
             src=loc!(d 4),
             blockers=loc!(f 4), loc!(f 6), loc!(b 2);
             attacks=loc!(b 4), loc!(b 6), loc!(d 6);
             moves=loc!(c 5),
                   loc!(e 5),
                   loc!(e 3),
                   loc!(f 2),
                   loc!(g 1),
                   loc!(c 3),
                   loc!(d 1),
                   loc!(d 2),
                   loc!(d 3),
                   loc!(d 5),
                   loc!(c 4),
                   loc!(e 4));
}
