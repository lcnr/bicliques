use bicliques::*;
use iai::black_box;
use std::ops::ControlFlow;

const T: bool = true;
const f: bool = false;

macro_rules! bench {
    ($($name:ident, $k:expr, $graph:expr);+$(;)?) => {
        $(fn $name() {
            const DATA: &[&[bool]] = $graph;

            let right = DATA[0].len();
            let mut g = Bigraph::new(DATA.len() as u32, right as u32);
            for (y, row) in DATA.iter().enumerate() {
                assert_eq!(row.len(), right);
                for (x, _) in row.iter().enumerate().filter(|&(_, &t)| t) {
                    g.add(Entry(x as u32, y as u32));
                }
            }

            biclique_covers::<(), _>(&g, $k, |c| {
                black_box(c);
                ControlFlow::Continue(())
            });
        })+

        iai::main!($($name),+);
    };
}

bench!(
    mini, 3, &[&[T, T], &[f, T], &[T, f]];
    difficult5, 4, &[
        &[T, T, T, T, f],
        &[T, T, T, f, T],
        &[T, T, f, T, T],
        &[T, f, T, T, T],
        &[f, T, T, T, T],
    ];
    nfaLEsynMIN, 4, &[
        &[T, T, T, T, f],
        &[T, T, f, T, T],
        &[T, f, T, T, f],
        &[T, T, T, T, T],
        &[f, T, f, T, T],
    ];
    difficult6, 5, &[
        &[T, T, T, T, T, f],
        &[T, T, T, T, f, T],
        &[T, T, T, f, T, T],
        &[T, T, f, T, T, T],
        &[T, f, T, T, T, T],
        &[f, T, T, T, T, T],
    ];
);
