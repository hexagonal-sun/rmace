use crate::{
    mmove::{Move, MoveBuilder},
    piece::{Piece, PieceKind},
    position::{locus::Locus, Position},
};

use super::rays::{
    calc_north_east_rays_moves, calc_north_west_rays_moves, calc_south_east_rays_moves,
    calc_south_west_rays_moves,
};

impl Position {
    pub fn calc_bishop_moves(&self, src: Locus) -> Vec<Move> {
        let p = Piece::new(PieceKind::Bishop, self.to_play);
        let our_pieces = self.all_pieces_for_colour(self.to_play);
        let their_pieces = self.all_pieces_for_colour(self.to_play.next());
        let mut moves = Vec::new();
        let builder = MoveBuilder::new(p, src);
        let blockers = self.blockers();

        for dst in ((calc_north_west_rays_moves(src, blockers)
            | calc_north_east_rays_moves(src, blockers)
            | calc_south_west_rays_moves(src, blockers)
            | calc_south_east_rays_moves(src, blockers))
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
