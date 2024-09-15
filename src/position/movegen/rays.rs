use paste::paste;

use crate::position::{
    bitboard::BitBoard,
    locus::{loc, Locus},
};

macro_rules! unwrap {
    ($e:expr $(,)*) => {
        match $e {
            ::core::option::Option::Some(x) => x,
            ::core::option::Option::None => panic!("Unarap a none"),
        }
    };
}

enum ScanDir {
    Forward,
    Backward,
}

const RANK_ONE: BitBoard = BitBoard::empty()
    .set_piece_at(loc!(a 1))
    .set_piece_at(loc!(b 1))
    .set_piece_at(loc!(c 1))
    .set_piece_at(loc!(d 1))
    .set_piece_at(loc!(e 1))
    .set_piece_at(loc!(f 1))
    .set_piece_at(loc!(g 1))
    .set_piece_at(loc!(h 1));

const RANK_EIGHT: BitBoard = BitBoard::empty()
    .set_piece_at(loc!(a 8))
    .set_piece_at(loc!(b 8))
    .set_piece_at(loc!(c 8))
    .set_piece_at(loc!(d 8))
    .set_piece_at(loc!(e 8))
    .set_piece_at(loc!(f 8))
    .set_piece_at(loc!(g 8))
    .set_piece_at(loc!(h 8));

const FILE_A: BitBoard = BitBoard::empty()
    .set_piece_at(loc!(a 1))
    .set_piece_at(loc!(a 2))
    .set_piece_at(loc!(a 3))
    .set_piece_at(loc!(a 4))
    .set_piece_at(loc!(a 5))
    .set_piece_at(loc!(a 6))
    .set_piece_at(loc!(a 7))
    .set_piece_at(loc!(a 8));

const FILE_H: BitBoard = BitBoard::empty()
    .set_piece_at(loc!(h 1))
    .set_piece_at(loc!(h 2))
    .set_piece_at(loc!(h 3))
    .set_piece_at(loc!(h 4))
    .set_piece_at(loc!(h 5))
    .set_piece_at(loc!(h 6))
    .set_piece_at(loc!(h 7))
    .set_piece_at(loc!(h 8));

const PERIMETER: BitBoard = BitBoard::empty()
    .or(FILE_A)
    .or(FILE_H)
    .or(RANK_ONE)
    .or(RANK_EIGHT);

macro_rules! mk_ray_move_fn {
    ($rays:ident, $scan_dir:expr) => {
        paste! {
            pub const fn [<calc_ $rays:lower _moves>](src: Locus, blockers: BitBoard) -> BitBoard {
                let src_ray = $rays[src.to_idx() as usize];
                let ray_blockers = src_ray.and(blockers);

                if ray_blockers.is_empty() {
                    return src_ray;
                }

                let first_blocker = match $scan_dir {
                    ScanDir::Forward => ray_blockers.first_idx_fwd(),
                    ScanDir::Backward => ray_blockers.first_idx_rev(),
                };

                src_ray.and($rays[first_blocker as usize].not())
            }
        }
    };
}

macro_rules! mk_ray {
    ($dir:ident, $scan_dir:ident, $occ_mask:ident) => {
        paste! {
            pub const [<$dir:upper _RAYS>]: [BitBoard; 64] = [<mk_ $dir _rays>]().0;
            const [<$dir:upper _OCC_RAYS>]: [BitBoard; 64] = [<mk_ $dir _rays>]().1;
            mk_ray_move_fn!([<$dir:upper _RAYS>], ScanDir::$scan_dir);
            const fn [<mk_ $dir _rays>]() -> ([BitBoard; 64], [BitBoard; 64]) {
                let mut table = [BitBoard::empty(); 64];
                let mut occ_table = [BitBoard::empty(); 64];

                let mut idx = 0;

                while idx < 64 {
                    let mut l = unwrap!(Locus::from_idx(idx));
                    loop {
                        match l.$dir() {
                            Some(new_l) => {
                                l = new_l;
                                table[idx as usize] = table[idx as usize].set_piece_at(l);
                            }
                            None => break,
                        }
                    }
                    occ_table[idx as usize] = table[idx as usize].and($occ_mask.not());
                    idx += 1;
                }

                (table, occ_table)
            }
        }
    };

    ($dir1:ident $dir2:ident, $scan_dir:ident, $occ_mask:ident) => {
        paste! {
            pub const [<$dir1:upper _ $dir2:upper _RAYS>]: [BitBoard; 64] = [<mk_ $dir1 _ $dir2 _rays>]().0;
            const [<$dir1:upper _ $dir2:upper _OCC_RAYS>]: [BitBoard; 64] = [<mk_ $dir1 _ $dir2 _rays>]().1;
            mk_ray_move_fn!([<$dir1:upper _ $dir2:upper _RAYS>], ScanDir::$scan_dir);
            const fn [<mk_ $dir1 _ $dir2 _rays>]() -> ([BitBoard; 64], [BitBoard; 64]) {
                let mut table = [BitBoard::empty(); 64];
                let mut occ_table = [BitBoard::empty(); 64];

                let mut idx = 0;

                while idx < 64 {
                    let mut l = unwrap!(Locus::from_idx(idx));
                    loop {
                        match l.$dir1() {
                            Some(new_l) => match new_l.$dir2() {
                                Some(new_l) => {
                                    l = new_l;
                                    table[idx as usize] = table[idx as usize].set_piece_at(l);
                                }
                                None => break,
                            },
                            None => break,
                        }
                    }
                    occ_table[idx as usize] = table[idx as usize].and($occ_mask.not());
                    idx += 1;
                }

                (table, occ_table)
            }
        }
    };
}

pub const ROOK_OCC_MASK: [BitBoard; 64] = calc_rook_occ_mask();
pub const BISHOP_OCC_MASK: [BitBoard; 64] = calc_bishop_occ_mask();

const fn calc_rook_occ_mask() -> [BitBoard; 64] {
    let mut ret = [BitBoard::empty(); 64];
    let mut idx = 0;

    while idx < 64 {
        ret[idx as usize] = ret[idx as usize]
            .or(NORTH_OCC_RAYS[idx])
            .or(EAST_OCC_RAYS[idx])
            .or(WEST_OCC_RAYS[idx])
            .or(SOUTH_OCC_RAYS[idx]);

        idx += 1;
    }

    ret
}

const fn calc_bishop_occ_mask() -> [BitBoard; 64] {
    let mut ret = [BitBoard::empty(); 64];
    let mut idx = 0;

    while idx < 64 {
        ret[idx as usize] = ret[idx as usize]
            .or(NORTH_EAST_OCC_RAYS[idx])
            .or(NORTH_WEST_OCC_RAYS[idx])
            .or(SOUTH_EAST_OCC_RAYS[idx])
            .or(SOUTH_WEST_OCC_RAYS[idx]);

        idx += 1;
    }

    ret
}

mk_ray!(north, Forward, RANK_EIGHT);
mk_ray!(east, Forward, FILE_H);
mk_ray!(south, Backward, RANK_ONE);
mk_ray!(west, Backward, FILE_A);
mk_ray!(north east, Forward, PERIMETER);
mk_ray!(north west, Forward, PERIMETER);
mk_ray!(south east, Backward, PERIMETER);
mk_ray!(south west, Backward, PERIMETER);

#[cfg(test)]
mod tests {
    use super::{calc_north_east_rays_moves, calc_north_rays_moves, calc_north_west_rays_moves};
    use crate::position::{
        bitboard::BitBoard,
        locus::loc,
        movegen::rays::{
            calc_east_rays_moves, calc_south_east_rays_moves, calc_south_rays_moves,
            calc_south_west_rays_moves, calc_west_rays_moves,
        },
    };

    macro_rules! mk_test {
        ($fn_name:ident, $ray_fn:ident, src=$src_loc:expr,
         blocker=$simple_piece_loc:expr, moves=$($simple_expected_locs:expr);+,
         blockers=$($multi_pieces:expr);+, moves=$($multi_expected_locs:expr);+) => {
            #[test]
            fn $fn_name() {
                let bb = BitBoard::empty().set_piece_at($simple_piece_loc);

                let bb = $ray_fn($src_loc, bb);

                assert_eq!(
                    bb,
                    BitBoard::empty()
                        $(.set_piece_at($simple_expected_locs))+
                );

                let bb = BitBoard::empty()
                    $(.set_piece_at($multi_pieces))+;

                let bb = $ray_fn($src_loc, bb);

                println!("{}", bb);

                assert_eq!(
                    bb,
                    BitBoard::empty()
                        $(.set_piece_at($multi_expected_locs))+
                );
            }
        };
    }

    mk_test!(north, calc_north_rays_moves,
             src=loc!(c 2),
             blocker=loc!(c 6),
             moves=loc!(c 3); loc!(c 4); loc!(c 5); loc!(c 6),
             blockers=loc!(c 6); loc!(d 4); loc!(c 4); loc!(c 5),
             moves=loc!(c 3); loc!(c 4));

    mk_test!(north_east, calc_north_east_rays_moves,
             src=loc!(c 2),
             blocker=loc!(f 5),
             moves=loc!(d 3); loc!(e 4); loc!(f 5),
             blockers=loc!(e 4); loc!(g 6); loc!(c 4); loc!(c 5),
             moves=loc!(d 3); loc!(e 4));

    mk_test!(east, calc_east_rays_moves,
             src=loc!(b 2),
             blocker=loc!(f 2),
             moves=loc!(c 2); loc!(d 2); loc!(e 2); loc!(f 2),
             blockers=loc!(d 2); loc!(g 6); loc!(f 2); loc!(g 2),
             moves=loc!(c 2); loc!(d 2));

    mk_test!(south_east, calc_south_east_rays_moves,
             src=loc!(b 7),
             blocker=loc!(e 4),
             moves=loc!(c 6); loc!(d 5); loc!(e 4),
             blockers=loc!(f 3); loc!(d 5); loc!(f 2); loc!(g 2),
             moves=loc!(c 6); loc!(d 5));

    mk_test!(south, calc_south_rays_moves,
             src=loc!(b 7),
             blocker=loc!(b 4),
             moves=loc!(b 6); loc!(b 5); loc!(b 4),
             blockers=loc!(b 4); loc!(b 2); loc!(f 2); loc!(g 2),
             moves=loc!(b 6); loc!(b 5); loc!(b 4));

    mk_test!(south_west, calc_south_west_rays_moves,
             src=loc!(g 7),
             blocker=loc!(d 4),
             moves=loc!(f 6); loc!(e 5); loc!(d 4),
             blockers=loc!(e 5); loc!(c 3); loc!(f 2); loc!(g 2),
             moves=loc!(f 6); loc!(e 5));

    mk_test!(west, calc_west_rays_moves,
             src=loc!(g 7),
             blocker=loc!(d 7),
             moves=loc!(f 7); loc!(e 7); loc!(d 7),
             blockers=loc!(e 7); loc!(c 7); loc!(f 2); loc!(g 2),
             moves=loc!(f 7); loc!(e 7));

    mk_test!(north_west, calc_north_west_rays_moves,
             src=loc!(g 2),
             blocker=loc!(d 5),
             moves=loc!(f 3); loc!(e 4); loc!(d 5),
             blockers=loc!(e 4); loc!(c 6); loc!(f 2),
             moves=loc!(f 3); loc!(e 4));
}
