use strum::{EnumCount, IntoEnumIterator};

use crate::piece::{Colour, Piece, PieceKind};

use super::{bitboard::BitBoard, Position};

pub struct Evaluator<'a> {
    pos: &'a Position,
}

type MatPoint = [i32; PieceKind::COUNT - 1];

const MATERIAL_POINTS: MatPoint = calc_material_points();

#[rustfmt::skip]
const PSQT_PAWN: [[i32; 64]; 2] = calc_table([
     0,  0,  0,  0,  0,  0,  0,  0, // 1
     5, 10, 10,-20,-20, 10, 10,  5, // 2
     5, -5,-10,  0,  0,-10, -5,  5, // 3
     0,  0,  0, 20, 20,  0,  0,  0, // 4
     5,  5, 10, 25, 25, 10,  5,  5, // 5
    10, 10, 20, 30, 30, 20, 10, 10, // 6
    50, 50, 50, 50, 50, 50, 50, 50, // 7
     0,  0,  0,  0,  0,  0,  0,  0, // 8
  // A     B     C     D     E     F     G     H
]);

#[rustfmt::skip]
const PSQT_KNIGHT: [[i32; 64]; 2] = calc_table([
    -50,-40,-30,-30,-30,-30,-40,-50, // 1
    -40,-20,  0,  5,  5,  0,-20,-40, // 2
    -30,  5, 10, 15, 15, 10,  5,-30, // 3
    -30,  0, 15, 20, 20, 15,  0,-30, // 4
    -30,  5, 15, 20, 20, 15,  5,-30, // 5
    -30,  0, 10, 15, 15, 10,  0,-30, // 6
    -40,-20,  0,  0,  0,  0,-20,-40, // 7
    -50,-40,-30,-30,-30,-30,-40,-50, // 8
  // A     B     C     D     E     F     G     H
]);

#[rustfmt::skip]
const PSQT_BISHOP: [[i32; 64]; 2] = calc_table([
    -20,-10,-10,-10,-10,-10,-10,-20, // 1
    -10,  5,  0,  0,  0,  0,  5,-10, // 2
    -10, 10, 10, 10, 10, 10, 10,-10, // 3
    -10,  0, 10, 10, 10, 10,  0,-10, // 4
    -10,  5,  5, 10, 10,  5,  5,-10, // 5
    -10,  0,  5, 10, 10,  5,  0,-10, // 6
    -10,  0,  0,  0,  0,  0,  0,-10, // 7
    -20,-10,-10,-10,-10,-10,-10,-20, // 8
  // A     B     C     D     E     F     G     H
]);

#[rustfmt::skip]
const PSQT_ROOK: [[i32; 64]; 2] = calc_table([
     0,  0,  0,  5,  5,  0,  0,  0, // 1
    -5,  0,  0,  0,  0,  0,  0, -5, // 2
    -5,  0,  0,  0,  0,  0,  0, -5, // 3
    -5,  0,  0,  0,  0,  0,  0, -5, // 4
    -5,  0,  0,  0,  0,  0,  0, -5, // 5
    -5,  0,  0,  0,  0,  0,  0, -5, // 6
     5, 10, 10, 10, 10, 10, 10,  5, // 7
     0,  0,  0,  0,  0,  0,  0,  0, // 8
  // A     B     C     D     E     F     G     H
]);

#[rustfmt::skip]
const PSQT_QUEEN: [[i32; 64]; 2] = calc_table([
    -20,-10,-10, -5, -5,-10,-10,-20, // 1
    -10,  0,  5,  0,  0,  0,  0,-10, // 2
    -10,  5,  5,  5,  5,  5,  0,-10, // 3
      0,  0,  5,  5,  5,  5,  0, -5, // 4
     -5,  0,  5,  5,  5,  5,  0, -5, // 5
    -10,  0,  5,  5,  5,  5,  0,-10, // 6
    -10,  0,  0,  0,  0,  0,  0,-10, // 7
    -20,-10,-10, -5, -5,-10,-10,-20, // 8
  // A     B     C     D     E     F     G     H
]);

#[rustfmt::skip]
const PSQT_KING_MIDDLE: [[i32; 64]; 2] = calc_table([
     20, 30, 10,  0,  0, 10, 30, 20, // 1
     20, 20,  0,  0,  0,  0, 20, 20, // 2
    -10,-20,-20,-20,-20,-20,-20,-10, // 3
    -20,-30,-30,-40,-40,-30,-30,-20, // 4
    -30,-40,-40,-50,-50,-40,-40,-30, // 5
    -30,-40,-40,-50,-50,-40,-40,-30, // 6
    -30,-40,-40,-50,-50,-40,-40,-30, // 7
    -30,-40,-40,-50,-50,-40,-40,-30, // 8
  // A     B     C     D     E     F     G     H
]);

#[rustfmt::skip]
const PSQT_KING_END: [[i32; 64]; 2] = calc_table([
    -50,-30,-30,-30,-30,-30,-30,-50, // 1
    -30,-30,  0,  0,  0,  0,-30,-30, // 2
    -30,-10, 20, 30, 30, 20,-10,-30, // 3
    -30,-10, 30, 40, 40, 30,-10,-30, // 4
    -30,-10, 30, 40, 40, 30,-10,-30, // 5
    -30,-10, 20, 30, 30, 20,-10,-30, // 6
    -30,-20,-10,  0,  0,-10,-20,-30, // 7
    -50,-40,-30,-20,-20,-30,-40,-50, // 8
  // A     B     C     D     E     F     G     H
]);

const fn flip(x: [i32; 64]) -> [i32; 64] {
    let mut ret = [0; 64];
    let mut i = 0;

    loop {
        if i == 64 {
            break;
        }
        let col = i % 8;
        let row = i / 8;

        ret[(7 - row) * 8 + col] = x[i];

        i += 1;
    }

    ret
}

const fn calc_table(x: [i32; 64]) -> [[i32; 64]; 2] {
    [x, flip(x)]
}

const fn calc_material_points() -> MatPoint {
    let mut ret = [0; PieceKind::COUNT - 1];

    ret[PieceKind::Pawn as usize] = 10;
    ret[PieceKind::Rook as usize] = 30;
    ret[PieceKind::Bishop as usize] = 30;
    ret[PieceKind::Knight as usize] = 50;
    ret[PieceKind::Queen as usize] = 90;

    ret
}

impl<'a> Evaluator<'a> {
    fn apply_psqt(bb: BitBoard, psqt: &[i32; 64]) -> i32 {
        bb.iter_pieces().map(|x| psqt[x.to_idx() as usize]).sum()
    }

    fn calc_phase_coef(material_count: u8) -> f32 {
        if material_count < 10 {
            0.0
        } else if material_count > 20 {
            1.0
        } else {
            (material_count as f32 - 10.0) / 10.0
        }
    }

    fn calc_psqt(&self) -> i32 {
        let mut ret = 0;

        macro_rules! psqt {
            ($k:ident, $table:ident) => {
                ret += Self::apply_psqt(
                    self.pos[Piece::new(PieceKind::$k, Colour::White)],
                    &$table[0],
                );
                ret -= Self::apply_psqt(
                    self.pos[Piece::new(PieceKind::$k, Colour::Black)],
                    &$table[1],
                );
            };
            ($k:ident, $table:ident, $coeff:expr) => {
                ret += (($coeff)
                    * Self::apply_psqt(
                        self.pos[Piece::new(PieceKind::$k, Colour::White)],
                        &$table[0],
                    ) as f32) as i32;
                ret -= (($coeff)
                    * Self::apply_psqt(
                        self.pos[Piece::new(PieceKind::$k, Colour::Black)],
                        &$table[1],
                    ) as f32) as i32;
            };
        }

        let game_phase = Self::calc_phase_coef(self.pos.material_count);

        psqt!(Pawn, PSQT_PAWN);
        psqt!(Rook, PSQT_ROOK);
        psqt!(Bishop, PSQT_BISHOP);
        psqt!(Knight, PSQT_KNIGHT);
        psqt!(Queen, PSQT_QUEEN);
        psqt!(King, PSQT_KING_MIDDLE, game_phase);
        psqt!(King, PSQT_KING_END, 1.0 - game_phase);

        ret
    }

    fn count_material(&self) -> i32 {
        let mut ret = 0;

        for kind in PieceKind::iter().filter(|x| *x != PieceKind::King) {
            ret += self.pos[Piece::new(kind, Colour::White)].popcount() as i32
                * MATERIAL_POINTS[kind as usize];
            ret -= self.pos[Piece::new(kind, Colour::Black)].popcount() as i32
                * MATERIAL_POINTS[kind as usize];
        }

        ret
    }

    fn do_eval(&self) -> i32 {
        let mut ret = 0;

        ret += self.count_material();
        ret += self.calc_psqt();

        ret
    }

    pub fn eval(pos: &'a Position) -> i32 {
        Self { pos }.do_eval()
    }
}

#[cfg(test)]
mod tests {
    use crate::position::eval::Evaluator;

    #[test]
    fn game_phase_coeff() {
        assert_eq!(Evaluator::calc_phase_coef(6), 0.0);
        assert_eq!(Evaluator::calc_phase_coef(25), 1.0);
        assert_eq!(Evaluator::calc_phase_coef(16), 0.6);
    }
}
