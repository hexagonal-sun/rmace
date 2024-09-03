use std::ops::{Index, IndexMut};

use crate::piece::Colour;

use super::locus::{File, Locus, Rank};

#[derive(Clone, Copy, PartialEq, Debug)]
pub struct CastlingRight {
    king_side: bool,
    queen_side: bool,
}

impl Default for CastlingRight {
    fn default() -> Self {
        Self {
            king_side: true,
            queen_side: true,
        }
    }
}

impl CastlingRight {
    pub fn empty() -> Self {
        Self {
            king_side: false,
            queen_side: false,
        }
    }

    pub fn has_any(self) -> bool {
        self.king_side || self.queen_side
    }

    pub fn clear_all(&mut self) {
        self.king_side = false;
        self.queen_side = false;
    }

    pub fn clear_for_loc(&mut self, loc: Locus) {
        let (r, f) = loc.to_rank_file();
        if !(r == Rank::One || r == Rank::Eight) {
            return;
        }
        match f {
            File::H => self.king_side = false,
            File::A => self.queen_side = false,
            _ => (),
        }
    }

    pub fn queen_side(self) -> bool {
        self.queen_side
    }

    pub fn king_side(self) -> bool {
        self.king_side
    }

    pub fn set_king_side(&mut self) {
        self.king_side = true;
    }

    pub fn set_queen_side(&mut self) {
        self.queen_side = true;
    }
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub struct CastlingRights([CastlingRight; 2]);

impl CastlingRights {
    pub fn clear(&mut self, c: Colour, loc: Locus) {
        let (r, f) = loc.to_rank_file();
        if r != match c {
            Colour::White => Rank::One,
            Colour::Black => Rank::Eight,
        } {
            return;
        }

        match f {
            File::H => self[c].king_side = false,
            File::A => self[c].queen_side = false,
            _ => (),
        }
    }

}

impl Index<Colour> for CastlingRights {
    type Output = CastlingRight;

    fn index(&self, index: Colour) -> &Self::Output {
        match index {
            Colour::White => &self.0[0],
            Colour::Black => &self.0[1],
        }
    }
}

impl IndexMut<Colour> for CastlingRights {
    fn index_mut(&mut self, index: Colour) -> &mut Self::Output {
        match index {
            Colour::White => &mut self.0[0],
            Colour::Black => &mut self.0[1],
        }
    }
}

impl Default for CastlingRights {
    fn default() -> Self {
        Self(Default::default())
    }
}

impl CastlingRights {
    pub fn empty() -> Self {
        Self([CastlingRight::empty(); 2])
    }
}
