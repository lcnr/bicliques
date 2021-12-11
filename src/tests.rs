use crate::*;

use std::cmp::Ordering;
use std::collections::HashSet;

fn all_solutions(g: &Bigraph, k: usize) -> HashSet<BicliqueCover> {
    let mut cliques = HashSet::new();
    enum Never {}
    biclique_covers::<Never, _>(&g, k, |c| {
        if g.is_maximal_cover(&c) {
            assert!(cliques.insert(c));
        }
        ControlFlow::Continue(())
    });
    cliques
}

fn check_solutions<const N: usize>(
    mut v: [&'static str; N],
    g: &Bigraph,
    solutions: &HashSet<BicliqueCover>,
) {
    let mut w = solutions.iter().map(|c| c.print(g)).collect::<Vec<_>>();
    v.sort();
    w.sort();

    let mut err = false;
    macro_rules! err {
        ($($t:tt)*) => {
            {
                err = true;
                eprintln!($($t)*);
            }
        }
    }

    match v.len().cmp(&w.len()) {
        Ordering::Equal => {}
        Ordering::Less => err!("at least {} unexpected elements", w.len() - v.len()),
        Ordering::Greater => err!("at least {} missing elements", v.len() - w.len()),
    }

    let mut v = v.iter().peekable();
    let mut w = w.iter().peekable();
    loop {
        match (v.peek(), w.peek()) {
            (Some(a), None) => err!("missing clique:    {}", a),
            (None, Some(b)) => err!("unexpected clique: {}", b),
            (Some(&&a), Some(b)) => match a.cmp(b.as_str()) {
                Ordering::Equal => {}
                Ordering::Less => {
                    err!("missing clique:    {}", a);
                    v.next();
                    continue;
                }
                Ordering::Greater => {
                    err!("unexpected clique: {}", b);
                    w.next();
                    continue;
                }
            },
            (None, None) => {
                if err {
                    panic!();
                } else {
                    return;
                }
            }
        }

        v.next();
        w.next();
    }
}

const T: bool = true;
const F: bool = false;

#[test]
fn small() {
    let g = Bigraph::from([[T, T], [F, T], [T, F]]);
    let solutions = all_solutions(&g, 5);
    check_solutions(["101|10 110|01", "101|10 110|01 100|11"], &g, &solutions)
}

#[test]
fn syn_le_min() {
    let g = Bigraph::from([
        [T, T, T, T, F],
        [T, T, F, T, T],
        [T, F, T, T, F],
        [T, T, T, T, T],
        [F, T, F, T, T],
    ]);
    let solutions = all_solutions(&g, 4);
    check_solutions(
        [
            "01011|01011 10110|10110 11010|11010",
            "01011|01011 10110|10110 11010|11010 10010|11110",
            "01011|01011 11110|10010 10110|10110 10010|11110",
            "01011|01011 10110|10110 01010|11011 10010|11110",
            "01011|01011 10110|10110 11010|11010 01010|11011",
            "01011|01011 10110|10110 11010|11010 00010|11111",
            "11011|01010 01011|01011 11110|10010 10110|10110",
            "11011|01010 01011|01011 10110|10110 01010|11011",
        ],
        &g,
        &solutions,
    )
}
