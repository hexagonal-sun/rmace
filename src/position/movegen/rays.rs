use crate::position::{bitboard::BitBoard, locus::Locus};

const NORTH_RAYS: [BitBoard; 64] = mk_north_rays();
const EAST_RAYS: [BitBoard; 64] = mk_east_rays();
const SOUTH_RAYS: [BitBoard; 64] = mk_south_rays();
const WEST_RAYS: [BitBoard; 64] = mk_west_rays();

const NE_RAYS: [BitBoard; 64] = mk_ne_rays();
const NW_RAYS: [BitBoard; 64] = mk_nw_rays();
const SW_RAYS: [BitBoard; 64] = mk_sw_rays();
const SE_RAYS: [BitBoard; 64] = mk_se_rays();

macro_rules! unwrap {
    ($e:expr $(,)*) => {
        match $e {
            ::core::option::Option::Some(x) => x,
            ::core::option::Option::None => panic!("Unarap a none"),
        }
    };
}

macro_rules! mk_striaght_ray {
    ($fn_name:ident, $dir:ident) => {
        const fn $fn_name() -> [BitBoard; 64] {
            let mut table = [BitBoard::empty(); 64];

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
                idx += 1;
            }

            table
        }
    };
}

macro_rules! mk_diag_ray {
    ($fn_name:ident, $dir1:ident, $dir2: ident) => {
        const fn $fn_name() -> [BitBoard; 64] {
            let mut table = [BitBoard::empty(); 64];

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
                idx += 1;
            }

            table
        }
    };
}

macro_rules! mk_blocker_fn {
    ($fn_name:ident, $rays:ident, $scan_dir:expr) => {
        pub const fn $fn_name(src: Locus, blockers: BitBoard) -> BitBoard {
            let src_ray = $rays[src.to_idx() as usize];

            if src_ray.is_empty() || blockers.is_empty() {
                return src_ray;
            }

            let ray_blockers = src_ray.and(blockers);

            let first_blocker = match $scan_dir {
                ScanDir::Forward => ray_blockers.first_idx_fwd(),
                ScanDir::Backward => ray_blockers.first_idx_rev(),
            };

            src_ray.and($rays[first_blocker as usize].not())
        }
    };
}

mk_striaght_ray!(mk_north_rays, north);
mk_striaght_ray!(mk_east_rays, east);
mk_striaght_ray!(mk_south_rays, south);
mk_striaght_ray!(mk_west_rays, west);
mk_diag_ray!(mk_ne_rays, north, east);
mk_diag_ray!(mk_nw_rays, north, west);
mk_diag_ray!(mk_se_rays, south, east);
mk_diag_ray!(mk_sw_rays, south, west);

enum ScanDir {
    Forward,
    Backward,
}

mk_blocker_fn!(calc_n_ray_moves, NORTH_RAYS, ScanDir::Forward);
mk_blocker_fn!(calc_ne_ray_moves, NE_RAYS, ScanDir::Forward);
mk_blocker_fn!(calc_e_ray_moves, EAST_RAYS, ScanDir::Forward);
mk_blocker_fn!(calc_se_ray_moves, SE_RAYS, ScanDir::Backward);
mk_blocker_fn!(calc_s_ray_moves, SOUTH_RAYS, ScanDir::Backward);
mk_blocker_fn!(calc_sw_ray_moves, SW_RAYS, ScanDir::Backward);
mk_blocker_fn!(calc_w_ray_moves, WEST_RAYS, ScanDir::Backward);
mk_blocker_fn!(calc_nw_ray_moves, NW_RAYS, ScanDir::Forward);

#[cfg(test)]
mod tests {
    use crate::position::{
        bitboard::BitBoard,
        locus::loc,
        movegen::rays::{
            calc_e_ray_moves, calc_n_ray_moves, calc_ne_ray_moves, calc_nw_ray_moves,
            calc_s_ray_moves, calc_se_ray_moves, calc_sw_ray_moves, calc_w_ray_moves,
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

    mk_test!(north, calc_n_ray_moves,
             src=loc!(C, Two),
             blocker=loc!(C, Six),
             moves=loc!(C, Three); loc!(C, Four); loc!(C, Five); loc!(C, Six),
             blockers=loc!(C, Six); loc!(D, Four); loc!(C, Four); loc!(C, Five),
             moves=loc!(C, Three); loc!(C, Four));

    mk_test!(north_east, calc_ne_ray_moves,
             src=loc!(C, Two),
             blocker=loc!(F, Five),
             moves=loc!(D, Three); loc!(E, Four); loc!(F, Five),
             blockers=loc!(E, Four); loc!(G, Six); loc!(C, Four); loc!(C, Five),
             moves=loc!(D, Three); loc!(E, Four));

    mk_test!(east, calc_e_ray_moves,
             src=loc!(B, Two),
             blocker=loc!(F, Two),
             moves=loc!(C, Two); loc!(D, Two); loc!(E, Two); loc!(F, Two),
             blockers=loc!(D, Two); loc!(G, Six); loc!(F, Two); loc!(G, Two),
             moves=loc!(C, Two); loc!(D, Two));

    mk_test!(south_east, calc_se_ray_moves,
             src=loc!(B, Seven),
             blocker=loc!(E, Four),
             moves=loc!(C, Six); loc!(D, Five); loc!(E, Four),
             blockers=loc!(F, Three); loc!(D, Five); loc!(F, Two); loc!(G, Two),
             moves=loc!(C, Six); loc!(D, Five));

    mk_test!(south, calc_s_ray_moves,
             src=loc!(B, Seven),
             blocker=loc!(B, Four),
             moves=loc!(B, Six); loc!(B, Five); loc!(B, Four),
             blockers=loc!(B, Four); loc!(B, Two); loc!(F, Two); loc!(G, Two),
             moves=loc!(B, Six); loc!(B, Five); loc!(B, Four));

    mk_test!(south_west, calc_sw_ray_moves,
             src=loc!(G, Seven),
             blocker=loc!(D, Four),
             moves=loc!(F, Six); loc!(E, Five); loc!(D, Four),
             blockers=loc!(E, Five); loc!(C, Three); loc!(F, Two); loc!(G, Two),
             moves=loc!(F, Six); loc!(E, Five));

    mk_test!(west, calc_w_ray_moves,
             src=loc!(G, Seven),
             blocker=loc!(D, Seven),
             moves=loc!(F, Seven); loc!(E, Seven); loc!(D, Seven),
             blockers=loc!(E, Seven); loc!(C, Seven); loc!(F, Two); loc!(G, Two),
             moves=loc!(F, Seven); loc!(E, Seven));

    mk_test!(north_west, calc_nw_ray_moves,
             src=loc!(G, Two),
             blocker=loc!(D, Five),
             moves=loc!(F, Three); loc!(E, Four); loc!(D, Five),
             blockers=loc!(E, Four); loc!(C, Six); loc!(F, Two),
             moves=loc!(F, Three); loc!(E, Four));
}
