use crate::position::{bitboard::BitBoard, locus::Locus};

pub const ATTACK_KNIGHT: [BitBoard; 64] = calc_attack_knight();

macro_rules! gen_knight_attack {
    ($func_name:ident, $m1:ident, $m2:ident, $m3:ident) => {
        const fn $func_name(locus: Locus) -> Option<BitBoard> {
            match locus.$m1() {
                Some(l) => match l.$m2() {
                    Some(l) => match l.$m3() {
                        Some(l) => return Some(l.to_bitboard()),
                        None => return None,
                    },
                    None => return None,
                },
                None => return None,
            }
        }
    };
}

gen_knight_attack!(gen_knight_attack_nnw, north, north, west);
gen_knight_attack!(gen_knight_attack_nne, north, north, east);
gen_knight_attack!(gen_knight_attack_een, east, east, north);
gen_knight_attack!(gen_knight_attack_ees, east, east, south);
gen_knight_attack!(gen_knight_attack_sse, south, south, east);
gen_knight_attack!(gen_knight_attack_ssw, south, south, west);
gen_knight_attack!(gen_knight_attack_wws, west, west, south);
gen_knight_attack!(gen_knight_attack_wwn, west, west, north);

const fn gen_knight_attacks(locus: Locus) -> BitBoard {
    BitBoard::empty()
        .opt_or(gen_knight_attack_nnw(locus))
        .opt_or(gen_knight_attack_nne(locus))
        .opt_or(gen_knight_attack_een(locus))
        .opt_or(gen_knight_attack_ees(locus))
        .opt_or(gen_knight_attack_sse(locus))
        .opt_or(gen_knight_attack_ssw(locus))
        .opt_or(gen_knight_attack_wws(locus))
        .opt_or(gen_knight_attack_wwn(locus))
}

const fn calc_attack_knight() -> [BitBoard; 64] {
    let mut table: [BitBoard; 64] = [BitBoard::empty(); 64];
    let mut idx = 0;

    while idx < 64 {
        match Locus::from_idx(idx as u8) {
            Some(l) => table[idx] = gen_knight_attacks(l),
            None => unreachable!()
        }
        idx += 1;
    }

    table
}
