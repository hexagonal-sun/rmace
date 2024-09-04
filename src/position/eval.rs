use strum::{EnumCount, IntoEnumIterator};

use crate::piece::{Colour, Piece, PieceKind};

use super::{bitboard::BitBoard, Position};

pub struct Evaluator<'a> {
    pos: &'a Position,
}

type MatPoint = [f64; PieceKind::COUNT - 1];

const MATERIAL_POINTS: MatPoint = calc_material_points();

#[rustfmt::skip]
const PSQT_PAWN: [[f64; 64]; 2] = calc_table([
     0.0,  0.0,  0.0,  0.0,  0.0,  0.0,  0.0,  0.0, // 1
     5.0, 10.0, 10.0,-20.0,-20.0, 10.0, 10.0,  5.0, // 2
     5.0, -5.0,-10.0,  0.0,  0.0,-10.0, -5.0,  5.0, // 3
     0.0,  0.0,  0.0, 20.0, 20.0,  0.0,  0.0,  0.0, // 4
     5.0,  5.0, 10.0, 25.0, 25.0, 10.0,  5.0,  5.0, // 5
    10.0, 10.0, 20.0, 30.0, 30.0, 20.0, 10.0, 10.0, // 6
    50.0, 50.0, 50.0, 50.0, 50.0, 50.0, 50.0, 50.0, // 7
     0.0,  0.0,  0.0,  0.0,  0.0,  0.0,  0.0,  0.0, // 8
  // A     B     C     D     E     F     G     H
]);

#[rustfmt::skip]
const PSQT_KNIGHT: [[f64; 64]; 2] = calc_table([
    -50.0,-40.0,-30.0,-30.0,-30.0,-30.0,-40.0,-50.0, // 1
    -40.0,-20.0,  0.0,  5.0,  5.0,  0.0,-20.0,-40.0, // 2
    -30.0,  5.0, 10.0, 15.0, 15.0, 10.0,  5.0,-30.0, // 3
    -30.0,  0.0, 15.0, 20.0, 20.0, 15.0,  0.0,-30.0, // 4
    -30.0,  5.0, 15.0, 20.0, 20.0, 15.0,  5.0,-30.0, // 5
    -30.0,  0.0, 10.0, 15.0, 15.0, 10.0,  0.0,-30.0, // 6
    -40.0,-20.0,  0.0,  0.0,  0.0,  0.0,-20.0,-40.0, // 7
    -50.0,-40.0,-30.0,-30.0,-30.0,-30.0,-40.0,-50.0, // 8
  // A     B     C     D     E     F     G     H
]);

#[rustfmt::skip]
const PSQT_BISHOP: [[f64; 64]; 2] = calc_table([
    -20.0,-10.0,-10.0,-10.0,-10.0,-10.0,-10.0,-20.0, // 1
    -10.0,  5.0,  0.0,  0.0,  0.0,  0.0,  5.0,-10.0, // 2
    -10.0, 10.0, 10.0, 10.0, 10.0, 10.0, 10.0,-10.0, // 3
    -10.0,  0.0, 10.0, 10.0, 10.0, 10.0,  0.0,-10.0, // 4
    -10.0,  5.0,  5.0, 10.0, 10.0,  5.0,  5.0,-10.0, // 5
    -10.0,  0.0,  5.0, 10.0, 10.0,  5.0,  0.0,-10.0, // 6
    -10.0,  0.0,  0.0,  0.0,  0.0,  0.0,  0.0,-10.0, // 7
    -20.0,-10.0,-10.0,-10.0,-10.0,-10.0,-10.0,-20.0, // 8
  // A     B     C     D     E     F     G     H
]);

#[rustfmt::skip]
const PSQT_ROOK: [[f64; 64]; 2] = calc_table([
     0.0,  0.0,  0.0,  5.0,  5.0,  0.0,  0.0,  0.0, // 1
    -5.0,  0.0,  0.0,  0.0,  0.0,  0.0,  0.0, -5.0, // 2
    -5.0,  0.0,  0.0,  0.0,  0.0,  0.0,  0.0, -5.0, // 3
    -5.0,  0.0,  0.0,  0.0,  0.0,  0.0,  0.0, -5.0, // 4
    -5.0,  0.0,  0.0,  0.0,  0.0,  0.0,  0.0, -5.0, // 5
    -5.0,  0.0,  0.0,  0.0,  0.0,  0.0,  0.0, -5.0, // 6
     5.0, 10.0, 10.0, 10.0, 10.0, 10.0, 10.0,  5.0, // 7
     0.0,  0.0,  0.0,  0.0,  0.0,  0.0,  0.0,  0.0, // 8
  // A     B     C     D     E     F     G     H
]);

#[rustfmt::skip]
const PSQT_QUEEN: [[f64; 64]; 2] = calc_table([
    -20.0,-10.0,-10.0, -5.0, -5.0,-10.0,-10.0,-20.0, // 1
    -10.0,  0.0,  5.0,  0.0,  0.0,  0.0,  0.0,-10.0, // 2
    -10.0,  5.0,  5.0,  5.0,  5.0,  5.0,  0.0,-10.0, // 3
      0.0,  0.0,  5.0,  5.0,  5.0,  5.0,  0.0, -5.0, // 4
     -5.0,  0.0,  5.0,  5.0,  5.0,  5.0,  0.0, -5.0, // 5
    -10.0,  0.0,  5.0,  5.0,  5.0,  5.0,  0.0,-10.0, // 6
    -10.0,  0.0,  0.0,  0.0,  0.0,  0.0,  0.0,-10.0, // 7
    -20.0,-10.0,-10.0, -5.0, -5.0,-10.0,-10.0,-20.0, // 8
  // A     B     C     D     E     F     G     H
]);

#[rustfmt::skip]
const PSQT_KING_MIDDLE: [[f64; 64]; 2] = calc_table([
     20.0, 30.0, 10.0,  0.0,  0.0, 10.0, 30.0, 20.0, // 1
     20.0, 20.0,  0.0,  0.0,  0.0,  0.0, 20.0, 20.0, // 2
    -10.0,-20.0,-20.0,-20.0,-20.0,-20.0,-20.0,-10.0, // 3
    -20.0,-30.0,-30.0,-40.0,-40.0,-30.0,-30.0,-20.0, // 4
    -30.0,-40.0,-40.0,-50.0,-50.0,-40.0,-40.0,-30.0, // 5
    -30.0,-40.0,-40.0,-50.0,-50.0,-40.0,-40.0,-30.0, // 6
    -30.0,-40.0,-40.0,-50.0,-50.0,-40.0,-40.0,-30.0, // 7
    -30.0,-40.0,-40.0,-50.0,-50.0,-40.0,-40.0,-30.0, // 8
  // A     B     C     D     E     F     G     H
]);

#[rustfmt::skip]
const PSQT_KING_END: [[f64; 64]; 2] = calc_table([
    -50.0,-30.0,-30.0,-30.0,-30.0,-30.0,-30.0,-50.0, // 1
    -30.0,-30.0,  0.0,  0.0,  0.0,  0.0,-30.0,-30.0, // 2
    -30.0,-10.0, 20.0, 30.0, 30.0, 20.0,-10.0,-30.0, // 3
    -30.0,-10.0, 30.0, 40.0, 40.0, 30.0,-10.0,-30.0, // 4
    -30.0,-10.0, 30.0, 40.0, 40.0, 30.0,-10.0,-30.0, // 5
    -30.0,-10.0, 20.0, 30.0, 30.0, 20.0,-10.0,-30.0, // 6
    -30.0,-20.0,-10.0,  0.0,  0.0,-10.0,-20.0,-30.0, // 7
    -50.0,-40.0,-30.0,-20.0,-20.0,-30.0,-40.0,-50.0, // 8
  // A     B     C     D     E     F     G     H
]);

const fn flip(x: [f64; 64]) -> [f64; 64] {
    let mut ret = [0.0; 64];
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

const fn calc_table(x: [f64; 64]) -> [[f64; 64]; 2] {
    [x, flip(x)]
}

const fn calc_material_points() -> MatPoint {
    let mut ret = [0.0; PieceKind::COUNT - 1];

    ret[PieceKind::Pawn as usize] = 10.0;
    ret[PieceKind::Rook as usize] = 30.0;
    ret[PieceKind::Bishop as usize] = 30.0;
    ret[PieceKind::Knight as usize] = 50.0;
    ret[PieceKind::Queen as usize] = 90.0;

    ret
}

impl<'a> Evaluator<'a> {
    fn apply_psqt(bb: BitBoard, psqt: &[f64; 64]) -> f64 {
        bb.iter_pieces().map(|x| psqt[x.to_idx() as usize]).sum()
    }

    fn calc_phase_coef(material_count: u8) -> f64 {
        if material_count < 10 {
            0.0
        } else if material_count > 20 {
            1.0
        } else {
            (material_count as f64 - 10.0) / 10.0
        }
    }

    fn calc_psqt(&self) -> f64 {
        let mut ret = 0.0;

        macro_rules! psqt {
            ($k:ident, $table:ident) => {
                psqt!($k, $table, 1.0)
            };
            ($k:ident, $table:ident, $coeff:expr) => {
                ret += ($coeff)
                    * Self::apply_psqt(
                        self.pos[Piece::new(PieceKind::$k, Colour::White)],
                        &$table[0],
                    );
                ret -= ($coeff)
                    * Self::apply_psqt(
                        self.pos[Piece::new(PieceKind::$k, Colour::Black)],
                        &$table[1],
                    );
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

    fn count_material(&self) -> f64 {
        let mut ret = 0.0;

        for kind in PieceKind::iter().filter(|x| *x != PieceKind::King) {
            ret += self.pos[Piece::new(kind, Colour::White)].popcount() as f64
                * MATERIAL_POINTS[kind as usize];
            ret -= self.pos[Piece::new(kind, Colour::Black)].popcount() as f64
                * MATERIAL_POINTS[kind as usize];
        }

        ret
    }

    fn do_eval(&self) -> f64 {
        let mut ret = 0.0;

        ret += self.count_material();
        ret += self.calc_psqt();

        ret
    }

    pub fn eval(pos: &'a Position) -> f64 {
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
