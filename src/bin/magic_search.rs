use itertools::Itertools;
use rand::random;
use rmace::position::{
    bitboard::BitBoard,
    movegen::rays::{BISHOP_OCC_MASK, ROOK_OCC_MASK},
};

fn search(v: u64, bbds: &Vec<BitBoard>, popcnt: u32, collisions: &mut Vec<bool>) -> bool {
    let shift = 63 - (popcnt - 1);
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

fn random_fewbits() -> u64 {
    random::<u64>() & random::<u64>() & random::<u64>()
}

fn search_ray(ray: BitBoard) {
    let popcnt = ray.popcount();

    eprintln!("{}", ray);
    eprintln!(
        "Popcount {}, looking for hash table of {}...",
        ray.popcount(),
        2u64.pow(ray.popcount())
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

    let mut collisions = vec![false; 1 << popcnt];

    loop {
        let n: u64 = random_fewbits();

        if search(n, &bbds, popcnt, &mut collisions) {
            println!("    ({popcnt}, 0x{n:X}),");
            eprintln!("Magic 0x{n:X} found!");
            return;
        }

        collisions.fill(false);
    }
}

fn main() {
    println!("const ROOK_MAGICS: [(usize, u64); 64] = {{");
    for i in 0..64 {
        search_ray(ROOK_OCC_MASK[i]);
    }
    println!("}}");

    println!("const BISHOP_MAGICS: [u64; 64] = {{");
    for i in 0..64 {
        eprintln!("BISHOP_OCC_MASK[{}]", i);
        search_ray(BISHOP_OCC_MASK[i]);
    }
    println!("}}");
}
