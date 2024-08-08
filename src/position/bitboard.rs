use std::{fmt::Display, ops::{BitAnd, Not}};

use strum::IntoEnumIterator;

use super::locus::{File, Rank};

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
}
