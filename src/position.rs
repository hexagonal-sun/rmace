use std::mem::variant_count;

use crate::piece::{Colour, PieceKind};

mod locus;
mod movegen;

#[derive(Clone, Copy, PartialEq)]
#[repr(transparent)]
pub struct BitBoard {
    inner: u64,
}

impl Display for BitBoard {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let b = format!("{:064b}", self.inner).into_bytes();
        for (b, rank) in b.chunks(8).zip(Rank::iter().rev()) {
            write!(f, "{:?} ", rank)?;
            for v in b.iter().rev() {
                write!(f, "{} ", match v {
                    b'0' => '.',
                    b'1' => '1',
                    _ => unreachable!()
                })?;
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

#[derive(Clone, PartialEq)]
struct Position {
    bboards: [BitBoard; variant_count::<PieceKind>() * 2],
    to_play: Colour,
}

impl Position {
    pub fn lookup(&self, colour: Colour, kind: PieceKind) -> BitBoard {
        self.bboards[colour as usize + kind as usize]
    }

    pub fn movegen(&self) -> Vec<Position> {
        todo!()
    }
}
