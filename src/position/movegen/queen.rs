use crate::{
    mmove::{Move, MoveBuilder},
    piece::{Piece, PieceKind},
    position::{locus::Locus, Position},
};

use super::rays::{
    calc_east_rays_moves, calc_north_east_rays_moves, calc_north_rays_moves,
    calc_north_west_rays_moves, calc_south_east_rays_moves, calc_south_rays_moves,
    calc_south_west_rays_moves, calc_west_rays_moves,
};

impl Position {
    pub fn calc_queen_moves(&self, src: Locus) -> Vec<Move> {
        let p = Piece::new(PieceKind::Queen, self.to_play);
        let our_pieces = self.all_pieces_for_colour(self.to_play);
        let their_pieces = self.all_pieces_for_colour(self.to_play.next());
        let mut moves = Vec::new();
        let builder = MoveBuilder::new(p, src);
        let blockers = self.blockers();

        for dst in ((calc_north_west_rays_moves(src, blockers)
            | calc_north_east_rays_moves(src, blockers)
            | calc_south_west_rays_moves(src, blockers)
            | calc_south_east_rays_moves(src, blockers)
            | calc_north_rays_moves(src, blockers)
            | calc_east_rays_moves(src, blockers)
            | calc_south_rays_moves(src, blockers)
            | calc_west_rays_moves(src, blockers))
            & (!our_pieces))
            .iter_pieces()
        {
            let mut m = builder.with_dst(dst);

            if their_pieces.has_piece_at(dst) {
                let piece = self
                    .piece_at_loc(dst)
                    .expect("their_pieces bb has piece at loc");

                m = m.with_capture(piece);
            }

            moves.push(m.build());
        }

        moves
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
