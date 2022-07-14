use exhaustigen::Gen;

use crate::*;

fn already_forced(g: &Bigraph, forced: &[Entry], e: Entry) -> bool {
    forced
        .iter()
        .any(|f: &Entry| g.get(Entry(e.0, f.1)) && g.get(Entry(f.0, e.1)))
}

pub(crate) fn forced_elements(g: &Bigraph) -> Vec<Entry> {
    let mut gen = Gen::new();

    let mut best = Vec::new();
    while !gen.done() {
        let mut forced = Vec::new();
        for e in g.entries() {
            if !already_forced(g, &forced, e) && gen.flip() {
                forced.push(e);
            }
        }

        if forced.len() > best.len() {
            best = forced;
        }
    }

    best
}
