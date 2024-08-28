use std::fmt::Debug;

use strum::{EnumCount, EnumIter};

use super::BitBoard;

macro_rules! rank {
    (1) => {
        crate::position::locus::Rank::One
    };
    (2) => {
        crate::position::locus::Rank::Two
    };
    (3) => {
        crate::position::locus::Rank::Three
    };
    (4) => {
        crate::position::locus::Rank::Four
    };
    (5) => {
        crate::position::locus::Rank::Five
    };
    (6) => {
        crate::position::locus::Rank::Six
    };
    (7) => {
        crate::position::locus::Rank::Seven
    };
    (8) => {
        crate::position::locus::Rank::Eight
    };
}

macro_rules! file {
    (a) => {
        crate::position::locus::File::A
    };
    (b) => {
        crate::position::locus::File::B
    };
    (c) => {
        crate::position::locus::File::C
    };
    (d) => {
        crate::position::locus::File::D
    };
    (e) => {
        crate::position::locus::File::E
    };
    (f) => {
        crate::position::locus::File::F
    };
    (g) => {
        crate::position::locus::File::G
    };
    (h) => {
        crate::position::locus::File::H
    };
}

macro_rules! loc {
    ($file:ident $rank:tt) => {
        crate::position::locus::Locus::from_rank_file(
            crate::position::locus::rank!($rank),
            crate::position::locus::file!($file),
        )
    };
}

pub(crate) use file;
pub(crate) use loc;
pub(crate) use rank;

#[derive(PartialEq, Clone, Copy)]
pub struct Locus {
    pos: i8,
}

impl Debug for Locus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let (rank, file) = self.to_rank_file();
        file.fmt(f)?;
        rank.fmt(f)?;

        Ok(())
    }
}

#[derive(EnumIter, Clone, Copy, PartialEq, EnumCount)]
pub enum Rank {
    One,
    Two,
    Three,
    Four,
    Five,
    Six,
    Seven,
    Eight,
}

impl Debug for Rank {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::One => write!(f, "1"),
            Self::Two => write!(f, "2"),
            Self::Three => write!(f, "3"),
            Self::Four => write!(f, "4"),
            Self::Five => write!(f, "5"),
            Self::Six => write!(f, "6"),
            Self::Seven => write!(f, "7"),
            Self::Eight => write!(f, "8"),
        }
    }
}

impl Rank {
    const fn from_idx(value: i8) -> Self {
        match value {
            0 => Self::One,
            1 => Self::Two,
            2 => Self::Three,
            3 => Self::Four,
            4 => Self::Five,
            5 => Self::Six,
            6 => Self::Seven,
            7 => Self::Eight,
            _ => unreachable!(),
        }
    }
}

#[derive(Debug, EnumIter, Clone, Copy, PartialEq)]
pub enum File {
    A,
    B,
    C,
    D,
    E,
    F,
    G,
    H,
}

impl File {
    const fn from_idx(value: i8) -> Self {
        match value {
            0 => Self::A,
            1 => Self::B,
            2 => Self::C,
            3 => Self::D,
            4 => Self::E,
            5 => Self::F,
            6 => Self::G,
            7 => Self::H,
            _ => unreachable!(),
        }
    }
}

impl Locus {
    pub const fn north(self) -> Option<Locus> {
        let pos: i8 = self.pos + 8;

        if pos >= 64 {
            None
        } else {
            Some(Self { pos })
        }
    }

    pub const fn south(self) -> Option<Locus> {
        let pos: i8 = self.pos - 8;

        if pos < 0 {
            None
        } else {
            Some(Self { pos })
        }
    }

    pub const fn east(self) -> Option<Locus> {
        let (_, file) = self.to_rank_file();

        match file {
            File::H => None,
            _ => Some(Self { pos: self.pos + 1 }),
        }
    }

    pub const fn west(self) -> Option<Locus> {
        let (_, file) = self.to_rank_file();

        match file {
            File::A => None,
            _ => Some(Self { pos: self.pos - 1 }),
        }
    }

    pub const fn from_idx(idx: u8) -> Option<Locus> {
        if idx >= 64 {
            None
        } else {
            Some(Self { pos: idx as i8 })
        }
    }

    pub const fn to_idx(self) -> u8 {
        self.pos as u8
    }

    pub const fn from_rank_file(rank: Rank, file: File) -> Locus {
        Self {
            pos: file as i8 + (rank as i8 * 8),
        }
    }

    pub const fn to_rank_file(self) -> (Rank, File) {
        let rank: Rank = Rank::from_idx(self.pos / 8);
        let file: File = File::from_idx(self.pos % 8);

        (rank, file)
    }

    pub const fn to_bitboard(self) -> BitBoard {
        BitBoard::new(1 << self.pos)
    }

    pub fn iter_all_squares() -> AllSquareIter {
        AllSquareIter { idx: 0 }
    }
}

pub struct AllSquareIter {
    idx: u8,
}

impl Iterator for AllSquareIter {
    type Item = Locus;

    fn next(&mut self) -> Option<Self::Item> {
        let ret = Locus::from_idx(self.idx);

        self.idx += 1;

        ret
    }
}

#[cfg(test)]
mod tests {
    use itertools::Itertools;
    use strum::IntoEnumIterator;

    use super::{File, Locus, Rank};

    #[test]
    fn idx_to_rf_to_idx_eq() {
        for idx in 0..64 {
            let l = Locus::from_idx(idx).unwrap();
            let (rank, file) = l.to_rank_file();
            let l2 = Locus::from_rank_file(rank, file);

            assert_eq!(l, l2);
        }
    }

    #[test]
    fn east_boundary() {
        for rank in Rank::iter() {
            let l = Locus::from_rank_file(rank, File::H);
            assert!(matches!(l.east(), None));
        }
    }

    #[test]
    fn east_move() {
        for (f1, f2) in File::iter().tuple_windows() {
            for rank in Rank::iter() {
                let l = Locus::from_rank_file(rank, f1);
                let (new_rank, new_file) = l.east().unwrap().to_rank_file();
                assert_eq!(new_rank, rank);
                assert_eq!(new_file, f2);
            }
        }
    }

    #[test]
    fn west_boundary() {
        for rank in Rank::iter() {
            let l = Locus::from_rank_file(rank, File::A);
            assert!(matches!(l.west(), None));
        }
    }

    #[test]
    fn west_move() {
        for (f1, f2) in File::iter().rev().tuple_windows() {
            for rank in Rank::iter() {
                let l = Locus::from_rank_file(rank, f1);
                let (new_rank, new_file) = l.west().unwrap().to_rank_file();
                assert_eq!(new_rank, rank);
                assert_eq!(new_file, f2);
            }
        }
    }

    #[test]
    fn south_boundary() {
        for file in File::iter() {
            let l = Locus::from_rank_file(Rank::One, file);
            assert!(matches!(l.south(), None));
        }
    }

    #[test]
    fn south_move() {
        for (r1, r2) in Rank::iter().rev().tuple_windows() {
            for file in File::iter() {
                let l = Locus::from_rank_file(r1, file);
                let (new_rank, new_file) = l.south().unwrap().to_rank_file();
                assert_eq!(new_rank, r2);
                assert_eq!(new_file, file);
            }
        }
    }

    #[test]
    fn noth_boundary() {
        for file in File::iter() {
            let l = Locus::from_rank_file(Rank::Eight, file);
            assert!(matches!(l.north(), None));
        }
    }

    #[test]
    fn north_move() {
        for (r1, r2) in Rank::iter().tuple_windows() {
            for file in File::iter() {
                let l = Locus::from_rank_file(r1, file);
                let (new_rank, new_file) = l.north().unwrap().to_rank_file();
                assert_eq!(new_rank, r2);
                assert_eq!(new_file, file);
            }
        }
    }
}
