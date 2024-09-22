use std::collections::HashMap;

use crate::{mmove::Move, position::zobrist::ZobristKey};

#[derive(Clone)]
pub enum EntryKind {
    Score(Move),
    Alpha,
    Beta,
}

#[derive(Clone)]
pub struct TEntry {
    pub hash: ZobristKey,
    pub depth: u32,
    pub kind: EntryKind,
    pub eval: i32,
}

const TABLE_SZ_MB: usize = 256;
const ENTRIES: usize = TABLE_SZ_MB * 1024 * 1024 / std::mem::size_of::<TEntry>();

#[derive(Clone)]
pub struct TTable {
    table: HashMap<ZobristKey, TEntry>,
}

impl TTable {
    pub fn new() -> Self {
        Self {
            table: HashMap::with_capacity(ENTRIES),
        }
    }

    pub fn lookup(&self, hash: ZobristKey) -> Option<&TEntry> {
        self.table.get(&hash)
    }

    pub fn insert(&mut self, entry: TEntry) {
        self.table.insert(entry.hash, entry);
    }
}
