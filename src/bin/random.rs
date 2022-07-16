use std::time::Instant;

use bicliques::{forced::*, Bigraph, Entry};
use rand::{distributions::Bernoulli, distributions::Distribution};

fn main() {
    let rng = &mut rand::thread_rng();

    for x in 10.. {
        for y in 10..=x {
            for p in [0.1, 0.3, 0.5, 0.7, 0.9] {
                let d = Bernoulli::new(p).unwrap();
                let mut new_time = 0.0;
                let mut old_time = 0.0;
                for _ in 0..100 {
                    let mut graph = Bigraph::new(x, y);
                    for x_pos in 0..x {
                        for y_pos in 0..y {
                            if d.sample(rng) {
                                graph.add(Entry(x_pos, y_pos));
                            }
                        }
                    }

                    let now = Instant::now();
                    let brute_size_old = forced_elements_old(&graph).len();
                    old_time += now.elapsed().as_secs_f64();

                    let now = Instant::now();
                    let brute_size_new = forced_elements(&graph).len();
                    new_time += now.elapsed().as_secs_f64();
                    assert_eq!(brute_size_old, brute_size_new);

                    let now = Instant::now();
                    let brute_size_new = forced_elements(&graph).len();
                    new_time += now.elapsed().as_secs_f64();

                    let now = Instant::now();
                    let brute_size_old = forced_elements_old(&graph).len();
                    old_time += now.elapsed().as_secs_f64();
                    assert_eq!(brute_size_old, brute_size_new);
                }

                new_time /= 2.0;
                old_time /= 2.0;
                let ratio = new_time / old_time;
                println!(
                    "{x}x{y} with p={p}: old={old_time:>7.3}, new={new_time:>7.3}, ratio={ratio}"
                );
            }
        }
    }
}