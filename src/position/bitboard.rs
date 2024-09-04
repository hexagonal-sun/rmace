use std::{
    fmt::Display,
    ops::{BitAnd, BitOr, Not},
};

use strum::IntoEnumIterator;

use super::locus::{File, Locus, Rank};

#[derive(Clone, Copy, PartialEq, Debug)]
#[repr(transparent)]
pub struct BitBoard {
    inner: u64,
}

impl BitOr for BitBoard {
    type Output = Self;

    fn bitor(self, rhs: Self) -> Self::Output {
        Self {
            inner: self.inner | rhs.inner,
        }
    }
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
            write!(f, "{} ", rank)?;
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

    pub const fn is_empty(self) -> bool {
        self.inner == 0
    }

    pub const fn first_idx_fwd(self) -> u32 {
        self.inner.trailing_zeros()
    }

    pub const fn first_idx_rev(self) -> u32 {
        63 - self.inner.leading_zeros()
    }

    // Until https://github.com/rust-lang/rust/issues/90080 is stablised
    pub const fn or(self, other: Self) -> Self {
        Self {
            inner: self.inner | other.inner,
        }
    }

    pub const fn popcount(self) -> u32 {
        self.inner.count_ones()
    }

    pub const fn opt_or(self, other: Option<Self>) -> Self {
        match other {
            Some(bb) => self.or(bb),
            None => self,
        }
    }

    pub const fn and(self, other: Self) -> Self {
        Self {
            inner: self.inner & other.inner,
        }
    }

    pub const fn not(self) -> Self {
        Self { inner: !self.inner }
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
        assert_eq!(iter.next(), Some(loc!(a 1)));
        assert_eq!(iter.next(), Some(loc!(c 1)));
        assert_eq!(iter.next(), Some(loc!(f 1)));
        assert_eq!(iter.next(), Some(loc!(h 1)));
        assert_eq!(iter.next(), Some(loc!(a 2)));
        assert_eq!(iter.next(), Some(loc!(h 2)));
        assert_eq!(iter.next(), Some(loc!(a 3)));
        assert_eq!(iter.next(), Some(loc!(h 8)));
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn piece_at() {
        let b = BitBoard { inner: 0b1 };
        assert!(b.has_piece_at(loc!(a 1)));
        assert!(!b.has_piece_at(loc!(a 2)));
        assert!(!b.has_piece_at(loc!(e 4)));

        let b = BitBoard { inner: 0b10 };
        assert!(!b.has_piece_at(loc!(a 1)));
        assert!(!b.has_piece_at(loc!(e 4)));
        assert!(b.has_piece_at(loc!(b 1)));

        let b = BitBoard { inner: 0b1000101 };
        assert!(b.has_piece_at(loc!(a 1)));
        assert!(!b.has_piece_at(loc!(b 1)));
        assert!(b.has_piece_at(loc!(c 1)));
        assert!(!b.has_piece_at(loc!(d 1)));
        assert!(!b.has_piece_at(loc!(e 1)));
        assert!(!b.has_piece_at(loc!(f 1)));
        assert!(b.has_piece_at(loc!(g 1)));
        assert!(!b.has_piece_at(loc!(h 1)));

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
        assert_eq!(b.clear_piece_at(loc!(c 1)).inner, 0b1000001);
    }

    #[test]
    fn idx_fwd() {
        let b = BitBoard { inner: 0b1010100 };
        assert_eq!(b.first_idx_fwd(), 2);

        let b = BitBoard { inner: 0b101010000 };
        assert_eq!(b.first_idx_fwd(), 4);
    }

    #[test]
    fn idx_rev() {
        let b = BitBoard {
            inner: 0b00001010100,
        };
        assert_eq!(b.first_idx_rev(), 6);

        let b = BitBoard {
            inner: 0b000101010000,
        };
        assert_eq!(b.first_idx_rev(), 8);
    }
}
