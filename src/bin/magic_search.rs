use std::{fs::File, io::Read};

use itertools::Itertools;
use rmace::position::{bitboard::BitBoard, movegen::rays::NORTH_RAYS};

fn search(v: u64, bbds: &Vec<BitBoard>, popcnt: u32) -> bool {
    let shift = 63 - (popcnt - 1);
    let mut collisions = vec![false; 1 << popcnt];
    for bb in bbds.iter() {
        let idx = ((u64::from(*bb).overflowing_mul(v).0) >> shift) as usize;
        if collisions[idx] == false {
            collisions[idx] = true;
        } else {
            return false;
        }
    }

    return true;
}

fn search_ray(ray: BitBoard) {
    let popcnt = ray.popcount();
    println!(
        "{} bit set.  Therefore, {} combinations",
        popcnt,
        2_u32.pow(popcnt)
    );

    let bit_positions = ray
        .iter_pieces()
        .map(|x| x.to_idx() as u8)
        .collect::<Vec<_>>();

    let bbds = bit_positions
        .iter()
        .powerset()
        .map(|x| x.iter().fold(0u64, |accum, x| accum | 1 << (**x) as u64))
        .map(|x| BitBoard::new(x))
        .collect::<Vec<_>>();

    assert_eq!(bbds.len(), 1 << popcnt);

    let mut f = File::open("/dev/urandom").unwrap();
    let mut buf = [0u8; 8];

    loop {
        f.read_exact(&mut buf).unwrap();
        let n = u64::from_le_bytes(buf);

        if search(n, &bbds, popcnt) {
            println!("Perfect hash found! 0x{n:X}");
            return;
        }
    }

}

fn main() {
    for i in 0..63 {
        println!("NORTH {}", i);
        search_ray(NORTH_RAYS[i]);
    }
}
