use std::{fmt::Display, ops::{BitAnd, Not}};

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
        Self { inner: self.inner & rhs.inner }
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

            write!(f, "\n")?;
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

    pub fn iter_pieces(self)  -> PiecesIterator {
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
        dbg!(self.bb.inner);
        let x = dbg!(self.bb.inner.trailing_zeros());

        let locus = Locus::from_idx(self.shift as u8 + x as u8)?;

        if x < 63 {
            self.bb.inner >>= dbg!(x + 1);
        } else {
            self.bb.inner = 0;
        }

        self.shift += x + 1;

        Some(locus)
    }
}

#[cfg(test)]
mod tests {
    use crate::position::locus::{self, loc, File, Locus, Rank};

    use super::BitBoard;

    #[test]
    fn piece_iter() {
        let b = BitBoard { inner: 0b1 };
        let mut iter = b.iter_pieces();
        assert_eq!(iter.next(), Some(Locus::from_rank_file(Rank::One, File::A)));
        assert_eq!(iter.next(), None);

        let b = BitBoard { inner: 0x8000000000000000 };
        let mut iter = b.iter_pieces();
        assert_eq!(iter.next(), Some(Locus::from_rank_file(Rank::Eight, File::H)));
        assert_eq!(iter.next(), None);

        let mut b = BitBoard { inner: 0b11000000110100101 };
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

}
