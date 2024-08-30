macro_rules! mk_test {
        (name=$name:ident, calc_fn=$calc:ident, kind=$p:ident,
         src=$src:expr, blockers=$($blockers:expr),*; attacks=$($attacks:expr),*;
         moves=$($moves:expr),+) => {
        #[test]
        fn $name() {
            let src = $src;
            let piece = mkp!(White, $p);
            let attacks = [
                $($attacks),*
            ];
            let moves = [
                $($moves),+
            ];
            let p = PositionBuilder::new()
                $(.with_piece_at(mkp!(White, Pawn), $blockers))*
                $(.with_piece_at(mkp!(Black, Pawn), $attacks))*
                .build();
            let calculated_moves = p.$calc(src);
            let mgen = MoveBuilder::new(piece, src);

            let mut bb = crate::position::bitboard::BitBoard::empty();

            for l in calculated_moves.iter() {
                bb = bb.set_piece_at(l.dst);
            }

            println!("{}", bb);

            println!("{:?}", calculated_moves);

            assert_eq!(calculated_moves.len(), moves.len() + attacks.len());
            for l in moves.iter() {
                let mmove = mgen.with_dst(*l).build();
                if !calculated_moves.contains(&mmove) {
                    panic!("Move {:?} is missing", mmove);
                }
            }

            for l in attacks.iter() {
                assert!(calculated_moves.contains(&mgen.with_dst(*l).with_capture(mkp!(Black, Pawn)).build()));
            }
        }
    };
}

pub(crate) use mk_test;
