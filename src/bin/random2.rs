use std::time::Instant;

use bicliques::{forced::forced_elements, old, Bigraph, Edge};
use rand::{distributions::Bernoulli, distributions::Distribution};

fn main() {
    let rng = &mut rand::thread_rng();

    for x in 12.. {
        for y in 12..=x {
            for p in [0.5] {
                let d = Bernoulli::new(p).unwrap();
                let mut new_time = 0.0;
                for _ in 0..1 {
                    let mut graph = Bigraph::new(x, y);
                    for x_pos in 0..x {
                        for y_pos in 0..y {
                            if d.sample(rng) {
                                graph.add(Edge(x_pos, y_pos));
                            }
                        }
                    }


                    let now = Instant::now();
                    let brute_size_new = forced_elements(&graph).len();
                    new_time += now.elapsed().as_secs_f64();
                }

                println!(
                    "{x}x{y} with p={p}: new={new_time:>7.3}"
                );
            }
        }
    }
}
