#![allow(unused)]

use bicliques::*;

use std::collections::HashSet;
use std::ops::ControlFlow;

fn from_str(s: &str) -> Bigraph {
    let mut iter = s.split_whitespace();
    let left = iter.next().unwrap().parse::<u32>().unwrap();
    let right = iter.next().unwrap().parse::<u32>().unwrap();
    let mut g = Bigraph::new(left, right);
    for x in 0..left {
        for y in 0..right {
            match iter.next().unwrap() {
                "1" => g.add(Edge(x, y)),
                "_" => (),
                e => panic!("unexpected: {}", e),
            }
        }
    }

    assert!(iter.next().is_none());

    g
}

const EX0: &str = "5 5
1 1 1 1 _
1 1 _ 1 1
1 _ 1 1 _
1 1 1 1 1
_ 1 _ 1 1
";

const EX1: &str = "7 7
1 1 1 1 1 _ 1
1 1 1 1 _ 1 1
1 1 1 _ 1 1 1
1 1 _ 1 1 1 1
1 _ 1 1 1 1 1
_ 1 1 1 1 1 1
1 1 1 1 1 1 1
";

const EX2: &str = "6 6
_ 1 1 1 1 1
1 _ 1 1 1 1
1 1 1 1 1 _
1 1 1 _ 1 1
1 1 1 1 _ 1
1 1 _ 1 1 1
";

const EX3: &str = "6 7
1 1 1 1 _ 1 1
1 1 1 _ 1 1 1
1 1 1 1 1 1 _
1 _ 1 1 1 1 1
1 1 1 1 1 _ 1
_ _ 1 1 1 1 1
";

const EX4: &str = "6 6
1 1 _ 1 1 1
1 _ 1 1 1 1
1 1 1 1 _ 1
_ 1 1 1 1 1
1 1 1 1 1 _
1 1 1 _ 1 1
";

const EX5: &str = "3 2
1 1
_ 1
1 _
";

const EX6: &str = "7 11
1 1 1 1 1 1 _ _ 1 1 1
1 1 1 1 _ 1 1 1 _ 1 1
1 1 1 1 1 _ 1 1 1 1 _
1 1 _ 1 1 1 1 1 _ 1 1
1 1 1 1 1 1 1 1 1 _ _
1 1 1 _ 1 1 1 _ 1 1 1
_ 1 1 1 _ 1 1 1 _ 1 1
";

const EX7: &str = "14 10
1 1 1 _ 1 1 1 _ 1 1 1 _ _ 1
1 _ _ 1 1 1 _ 1 1 1 1 1 _ 1
_ _ 1 1 1 _ 1 _ 1 1 1 _ 1 _
1 1 1 _ 1 _ _ _ 1 1 1 1 1 _
1 1 1 _ 1 1 _ _ _ _ 1 _ 1 1
1 _ _ 1 _ _ _ _ 1 1 1 _ _ 1
_ 1 1 1 1 1 1 1 1 _ _ 1 1 _
1 _ 1 1 1 1 1 _ _ 1 1 _ _ 1
1 1 _ _ _ 1 1 _ 1 1 _ _ 1 1
1 1 1 1 1 _ 1 _ _ 1 1 1 1 1
";

const EX8: &str = "7 7
1 1 1 1 1 1 _
1 1 1 1 1 _ 1
1 1 1 1 _ 1 1
1 1 1 _ 1 1 1
1 1 _ 1 1 1 1
1 _ 1 1 1 1 1
_ 1 1 1 1 1 1
";

const EX9: &str = "6 6
1 1 1 1 1 _
1 1 1 1 _ 1
1 1 1 _ 1 1
1 1 _ 1 1 1
1 _ 1 1 1 1
_ 1 1 1 1 1
";

const EX10: &str = "6 6
1 1 1 1 1 _
1 1 1 1 _ 1
1 _ 1 _ _ _
1 1 _ 1 _ 1
1 _ 1 1 1 1
_ 1 1 1 1 1
";

const EX11: &str = "16 11
1 1 1 _ 1 1 1 _ 1 1 1 _ _ 1 1 1
1 _ _ 1 1 1 _ 1 1 1 1 1 _ 1 1 _
_ _ 1 1 1 _ 1 _ 1 1 1 _ 1 _ _ 1
1 1 1 _ 1 _ _ _ 1 1 1 1 1 _ 1 1
1 1 1 _ 1 1 _ _ _ _ 1 _ 1 1 _ _
1 _ _ 1 _ _ _ _ 1 1 1 _ _ 1 _ 1
_ 1 1 1 1 1 1 1 1 _ _ 1 1 _ _ _
1 _ 1 1 1 1 1 _ _ 1 1 _ _ 1 _ 1
1 1 _ _ _ 1 1 _ 1 1 _ _ 1 1 _ _
1 1 1 1 1 _ 1 _ _ 1 1 1 1 1 _ 1
1 1 _ 1 _ 1 1 1 _ _ 1 1 1 1 1 1
";

fn main() {
    let g = from_str(EX11);

    println!("{:?}", g);

    let mut i = 0;
    let mut cliques = HashSet::new();
    enum Never {}
    bicliques::biclique_covers::<Never, _>(&g, g.left().min(g.right()) as usize - 1, |mut c| {
        if g.is_maximal_cover(&c) {
            if cliques.is_empty() {
                println!("min: {}", c.cliques().len());
            }
            // println!("{}", c.print(&g));
            assert!(cliques.insert(c));
            i += 1;
        }
        ControlFlow::Continue(())
    });
    println!("{}", i);
}
