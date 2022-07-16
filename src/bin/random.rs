use std::time::Instant;

use bicliques::{forced::forced_elements, Bigraph, Entry};
use rand::{distributions::Bernoulli, distributions::Distribution};

fn main() {
    let rng = &mut rand::thread_rng();
    let d = Bernoulli::new(0.5).unwrap();

    for x in 10.. {
        for y in 10..=x {
            let mut graph = Bigraph::new(x, y);
            for x_pos in 0..x {
                for y_pos in 0..y {
                    if d.sample(rng) {
                        graph.add(Entry(x_pos, y_pos));
                    }
                }
            }

            let now = Instant::now();
            let brute_size_new = forced_elements(&graph).len();
            println!(
                "({x}, {y}): {brute_size_new} in {}",
                now.elapsed().as_secs_f64()
            );
        }
    }
}
