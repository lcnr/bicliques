use bicliques::*;

fn from_str(s: &str) -> Bigraph {
    let mut iter = s.split_whitespace();
    let left = iter.next().unwrap().parse::<u32>().unwrap();
    let right = iter.next().unwrap().parse::<u32>().unwrap();
    let mut g = Bigraph::new(left, right);
    for y in 0..right {
        for x in 0..left {
            match iter.next().unwrap() {
                "1" => g.add(Entry(x, y)),
                "0" => (),
                e => panic!("unexpected: {}", e),
            }
        }
    }

    assert!(iter.next().is_none());

    g
}

const INPUT: &'static str = "5 5
    1 1 1 1 0
    1 1 0 1 1
    1 0 1 1 0
    1 1 1 1 1
    0 1 0 1 1
";

fn main() {
    let g = from_str(INPUT);

    println!("{:?}", g);

    for c in bicliques::biclique_covers(&g, 3) {
        println!("{:?}", c);
    }
}
