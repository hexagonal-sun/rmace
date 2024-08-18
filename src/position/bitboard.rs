use std::{
    fmt::Display,
    ops::{BitAnd, Not},
};

use strum::IntoEnumIterator;

use super::locus::{File, Locus, Rank};

#[derive(Clone, Copy, PartialEq)]
#[repr(transparent)]
pub struct BitBoard {
    inner: u64,
}

impl BitAnd for BitBoard {
    type Output = Self;

    fn bitand(self, rhs: Self) -> Self::Output {
        Self {
            inner: self.inner & rhs.inner,
        }
    }
}

impl Not for BitBoard {
    type Output = Self;

    fn not(self) -> Self::Output {
        Self { inner: !self.inner }
    }
}

impl Display for BitBoard {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let b = format!("{:064b}", self.inner).into_bytes();
        for (b, rank) in b.chunks(8).zip(Rank::iter().rev()) {
            write!(f, "{:?} ", rank)?;
            for v in b.iter().rev() {
                write!(
                    f,
                    "{} ",
                    match v {
                        b'0' => '.',
                        b'1' => '1',
                        _ => unreachable!(),
                    }
                )?;
            }

            writeln!(f)?;
        }

        write!(f, "  ")?;
        for file in File::iter() {
            write!(f, "{:?} ", file)?;
        }

        Ok(())
    }
}

impl BitBoard {
    pub const fn new(value: u64) -> Self {
        Self { inner: value }
    }

    pub const fn empty() -> Self {
        Self { inner: 0 }
    }

    pub const fn or(self, other: Self) -> Self {
        Self {
            inner: self.inner | other.inner,
        }
    }

    pub const fn opt_or(self, other: Option<Self>) -> Self {
        match other {
            Some(bb) => self.or(bb),
            None => self,
        }
    }

    pub const fn has_piece_at(self, loc: Locus) -> bool {
        self.inner & 1 << loc.to_idx() != 0
    }

    pub const fn clear_piece_at(self, loc: Locus) -> Self {
        Self::new(self.inner & !(loc.to_bitboard().inner))
    }

    pub const fn set_piece_at(self, loc: Locus) -> Self {
        Self::new(self.inner | loc.to_bitboard().inner)
    }

    pub fn iter_pieces(self) -> PiecesIterator {
        PiecesIterator { bb: self, shift: 0 }
    }
}

pub struct PiecesIterator {
    bb: BitBoard,
    shift: u32,
}

impl Iterator for PiecesIterator {
    type Item = Locus;

    fn next(&mut self) -> Option<Self::Item> {
        let x = self.bb.inner.trailing_zeros();

        let locus = Locus::from_idx(self.shift as u8 + x as u8)?;

        if x < 63 {
            self.bb.inner >>= x + 1;
        } else {
            self.bb.inner = 0;
        }

        self.shift += x + 1;

        Some(locus)
    }
}

#[cfg(test)]
mod tests {
    use strum::IntoEnumIterator;

    use crate::position::locus::{loc, File, Locus, Rank};

    use super::BitBoard;

    #[test]
    fn piece_iter() {
        let b = BitBoard { inner: 0b1 };
        let mut iter = b.iter_pieces();
        assert_eq!(iter.next(), Some(Locus::from_rank_file(Rank::One, File::A)));
        assert_eq!(iter.next(), None);

        let b = BitBoard {
            inner: 0x8000000000000000,
        };
        let mut iter = b.iter_pieces();
        assert_eq!(
            iter.next(),
            Some(Locus::from_rank_file(Rank::Eight, File::H))
        );
        assert_eq!(iter.next(), None);

        let mut b = BitBoard {
            inner: 0b11000000110100101,
        };
        b.inner |= 0x8000000000000000;

        let mut iter = b.iter_pieces();
        assert_eq!(iter.next(), Some(loc!(A, One)));
        assert_eq!(iter.next(), Some(loc!(C, One)));
        assert_eq!(iter.next(), Some(loc!(F, One)));
        assert_eq!(iter.next(), Some(loc!(H, One)));
        assert_eq!(iter.next(), Some(loc!(A, Two)));
        assert_eq!(iter.next(), Some(loc!(H, Two)));
        assert_eq!(iter.next(), Some(loc!(A, Three)));
        assert_eq!(iter.next(), Some(loc!(H, Eight)));
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn piece_at() {
        let b = BitBoard { inner: 0b1 };
        assert!(b.has_piece_at(loc!(A, One)));
        assert!(!b.has_piece_at(loc!(A, Two)));
        assert!(!b.has_piece_at(loc!(E, Four)));

        let b = BitBoard { inner: 0b10 };
        assert!(!b.has_piece_at(loc!(A, One)));
        assert!(!b.has_piece_at(loc!(E, Four)));
        assert!(b.has_piece_at(loc!(B, One)));

        let b = BitBoard { inner: 0b1000101 };
        assert!(b.has_piece_at(loc!(A, One)));
        assert!(!b.has_piece_at(loc!(B, One)));
        assert!(b.has_piece_at(loc!(C, One)));
        assert!(!b.has_piece_at(loc!(D, One)));
        assert!(!b.has_piece_at(loc!(E, One)));
        assert!(!b.has_piece_at(loc!(F, One)));
        assert!(b.has_piece_at(loc!(G, One)));
        assert!(!b.has_piece_at(loc!(H, One)));

        for file in File::iter() {
            assert!(!b.has_piece_at(Locus::from_rank_file(Rank::Two, file)));
            assert!(!b.has_piece_at(Locus::from_rank_file(Rank::Three, file)));
            assert!(!b.has_piece_at(Locus::from_rank_file(Rank::Four, file)));
            assert!(!b.has_piece_at(Locus::from_rank_file(Rank::Five, file)));
            assert!(!b.has_piece_at(Locus::from_rank_file(Rank::Six, file)));
            assert!(!b.has_piece_at(Locus::from_rank_file(Rank::Seven, file)));
            assert!(!b.has_piece_at(Locus::from_rank_file(Rank::Eight, file)));
        }
    }

    #[test]
    fn clear_piece() {
        let b = BitBoard { inner: 0b1000101 };
        assert_eq!(b.clear_piece_at(loc!(C, One)).inner, 0b1000001);
    }
}
