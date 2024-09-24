use std::sync::LazyLock;

use itertools::Itertools;

use crate::position::{
    bitboard::BitBoard,
    locus::Locus,
    movegen::rays::{
        calc_north_east_rays_moves, calc_north_west_rays_moves, calc_south_east_rays_moves,
        calc_south_west_rays_moves, BISHOP_OCC_MASK,
    },
};

use super::rays::{
    calc_east_rays_moves, calc_north_rays_moves, calc_south_rays_moves, calc_west_rays_moves,
    ROOK_OCC_MASK,
};

const ROOK_SHIFTS: [usize; 64] = [
    12, 11, 11, 11, 11, 11, 11, 12, 11, 10, 10, 10, 10, 10, 10, 11, 11, 10, 10, 10, 10, 10, 10, 11,
    11, 10, 10, 10, 10, 10, 10, 11, 11, 10, 10, 10, 10, 10, 10, 11, 11, 10, 10, 10, 10, 10, 10, 11,
    11, 10, 10, 10, 10, 10, 10, 11, 12, 11, 11, 11, 11, 11, 11, 12,
];

const ROOK_MAGICS: [u64; 64] = [
    0x80003060804008,
    0x100208411004000,
    0x2000A1040820020,
    0x100100020450008,
    0x1A002200040810A0,
    0x900088500021400,
    0x8100028200210004,
    0x180008000204100,
    0x200A002081044200,
    0x8404401004406004,
    0x51B9002002410030,
    0x89A003042000820,
    0x4080800C00812800,
    0x4082001002000429,
    0x400C000401502A08,
    0x5880802100004080,
    0x481461800C400084,
    0x9100888020004000,
    0x2810110020030041,
    0x2020010482040,
    0x1010808008000400,
    0x10808042000400,
    0x1010100020004,
    0x185020000830844,
    0x2A0E802080004008,
    0x1200080804000,
    0x210448200120020,
    0x1012100100448,
    0x8400080100050010,
    0x4002C0801104020,
    0xA0C4104400120108,
    0x100040200028043,
    0x40004012A1800084,
    0x100A00040401002,
    0x2004822000801000,
    0x1080200A02001041,
    0xA18008008800400,
    0x1002001002000418,
    0x2001001C01000200,
    0x810009004E000084,
    0x9011249040008000,
    0x8040022000808042,
    0x1009420080120022,
    0x610021101090020,
    0x4040008008080,
    0x811000400090012,
    0x885019040042,
    0x4008420560001,
    0x1004801048210100,
    0x4804000610300,
    0x820B104100200100,
    0x501100080480080,
    0x280004110100,
    0x54010040020040,
    0x1006000108040600,
    0x210084024200,
    0x80044010A0800101,
    0x2029001480C001,
    0x42000401021000D,
    0x120100009001D,
    0x2002088100402,
    0x1003400020801,
    0x80122104084,
    0x180402400428106,
];

const BISHOP_SHIFTS: [usize; 64] = [
    6, 5, 5, 5, 5, 5, 5, 6, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 7, 7, 7, 7, 5, 5, 5, 5, 7, 9, 9, 7, 5, 5,
    5, 5, 7, 9, 9, 7, 5, 5, 5, 5, 7, 7, 7, 7, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 6, 5, 5, 5, 5, 5, 5, 6,
];

const BISHOP_MAGICS: [u64; 64] = [
    0x222840808005080,
    0x20082E08802020,
    0xC028218400800480,
    0x200808C300000800,
    0x8004242090800089,
    0x4460880540040066,
    0x42101C200001,
    0x80C40047103094,
    0x8038400404240846,
    0x1800101006008064,
    0x4003100122002401,
    0xA012382290582004,
    0x280020211062810,
    0x1000051320100000,
    0x4000004410041100,
    0x200019208808A840,
    0xCC1050C4104420,
    0x624001204740400,
    0x10050806405020,
    0x8800A12040C109,
    0x1020820081000,
    0x1002082414013,
    0x402008111132000,
    0x40910104011300,
    0xA0081010020805,
    0x4084200011021080,
    0x2021131080600,
    0x20080049004208,
    0x60B01000010C000,
    0xC00C0500829000A0,
    0x100084204E011400,
    0x2004C1810111880A,
    0x1E1084000481120,
    0x8202021BA00800,
    0x440024040808C1,
    0x808020280480080,
    0x408020400021100,
    0x4040022041008,
    0x82208400A11401,
    0x500A040440030040,
    0x400610046010CC00,
    0x840120241800,
    0x220030002201,
    0x2200002011040801,
    0x10040810120200,
    0x2020111204200200,
    0x805044C040089,
    0x9004040882040060,
    0x8420424814400040,
    0xD41030801044200,
    0x41010840C060040,
    0x2480410484041001,
    0x12000100A0A0100,
    0x8040052004090000,
    0x9220094214840000,
    0x9480800404100,
    0xC006020043049004,
    0x104011400820800,
    0x402008901881400,
    0x10200880840420,
    0x88010004100A0608,
    0x8088801210300120,
    0x100600230020084,
    0x808020812450204,
];

#[derive(PartialEq)]
pub enum MagicKind {
    Rook,
    Bishop,
}

pub struct Magics {
    tables: [Vec<BitBoard>; 64],
    magics: &'static [u64; 64],
    shifts: &'static [usize; 64],
    occ_mask: &'static [BitBoard; 64],
}

impl Magics {
    #[inline(always)]
    fn idx(blockers: BitBoard, magic: u64, shift: usize) -> usize {
        ((u64::from(blockers).overflowing_mul(magic).0) >> (64 - shift)) as usize
    }

    pub fn new(kind: MagicKind) -> Self {
        let mut tables = [const { Vec::new() }; 64];
        let occ_mask = match kind {
            MagicKind::Rook => &ROOK_OCC_MASK,
            MagicKind::Bishop => &BISHOP_OCC_MASK,
        };
        for loc in Locus::iter_all_squares() {
            let idx = loc.to_idx() as usize;
            let magics = match kind {
                MagicKind::Rook => &ROOK_MAGICS,
                MagicKind::Bishop => &BISHOP_MAGICS,
            };
            let shift = match kind {
                MagicKind::Rook => &ROOK_SHIFTS,
                MagicKind::Bishop => &BISHOP_SHIFTS,
            };
            let magic = magics[idx];
            let shift = shift[idx];
            let mut bbds = vec![BitBoard::empty(); 1 << shift];
            let mask_bit_positions = occ_mask[idx]
                .iter_pieces()
                .map(|x| x.to_idx() as usize)
                .collect::<Vec<_>>();
            let blockers = mask_bit_positions
                .iter()
                .powerset()
                .map(|x| x.iter().fold(0, |accum, x| accum | 1 << *x))
                .map(|x| BitBoard::new(x))
                .collect::<Vec<_>>();

            for blocker in blockers {
                let bb = match kind {
                    MagicKind::Rook => calc_north_rays_moves(loc, blocker)
                        .or(calc_east_rays_moves(loc, blocker))
                        .or(calc_south_rays_moves(loc, blocker))
                        .or(calc_west_rays_moves(loc, blocker)),
                    MagicKind::Bishop => calc_north_east_rays_moves(loc, blocker)
                        .or(calc_north_west_rays_moves(loc, blocker))
                        .or(calc_south_east_rays_moves(loc, blocker))
                        .or(calc_south_west_rays_moves(loc, blocker)),
                };

                bbds[Self::idx(blocker, magic, shift)] = bb;
            }
            tables[loc.to_idx() as usize] = bbds;
        }
        Self {
            tables,
            magics: match kind {
                MagicKind::Rook => &ROOK_MAGICS,
                MagicKind::Bishop => &BISHOP_MAGICS,
            },
            shifts: match kind {
                MagicKind::Rook => &ROOK_SHIFTS,
                MagicKind::Bishop => &BISHOP_SHIFTS,
            },
            occ_mask,
        }
    }

    #[inline(always)]
    pub fn lookup(&self, loc: Locus, blockers: BitBoard) -> BitBoard {
        let blockers = blockers.and(self.occ_mask[loc.to_idx() as usize]);
        let magic = self.magics[loc.to_idx() as usize];
        let shift = self.shifts[loc.to_idx() as usize];
        self.tables[loc.to_idx() as usize][Self::idx(blockers, magic, shift)]
    }
}

pub static BISHOP_TABLES: LazyLock<Magics> = LazyLock::new(|| Magics::new(MagicKind::Bishop));
pub static ROOK_TABLES: LazyLock<Magics> = LazyLock::new(|| Magics::new(MagicKind::Rook));

#[cfg(test)]
mod tests {
    use crate::position::{
        bitboard::BitBoard,
        locus::loc,
        movegen::magics::{BISHOP_TABLES, ROOK_TABLES},
    };

    #[test]
    fn magic_rook() {
        let blockers = BitBoard::empty()
            .set_piece_at(loc!(g 4))
            .set_piece_at(loc!(e 3))
            .set_piece_at(loc!(c 4));
        let loc = loc!(e 4);

        assert_eq!(
            ROOK_TABLES.lookup(loc, blockers),
            BitBoard::empty()
                .set_piece_at(loc!(e 3))
                .set_piece_at(loc!(d 4))
                .set_piece_at(loc!(f 4))
                .set_piece_at(loc!(g 4))
                .set_piece_at(loc!(e 5))
                .set_piece_at(loc!(e 6))
                .set_piece_at(loc!(e 7))
                .set_piece_at(loc!(e 8))
                .set_piece_at(loc!(c 4))
        );
    }

    #[test]
    fn magic_bishop() {
        let blockers = BitBoard::empty()
            .set_piece_at(loc!(f 5))
            .set_piece_at(loc!(g 6))
            .set_piece_at(loc!(b 3));
        let loc = loc!(c 2);

        assert_eq!(
            BISHOP_TABLES.lookup(loc, blockers),
            BitBoard::empty()
                .set_piece_at(loc!(b 3))
                .set_piece_at(loc!(b 1))
                .set_piece_at(loc!(d 3))
                .set_piece_at(loc!(d 1))
                .set_piece_at(loc!(e 4))
                .set_piece_at(loc!(f 5))
        );
    }
}
