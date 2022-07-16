use crate::*;

pub fn forced_elements(g: &Bigraph) -> Vec<Entry> {
    let mut mapping: Vec<_> = g.entries().collect();
    mapping.sort_by_cached_key(|&e| g.entries().filter(|&o| g.may_share(e, o)).count());
    let mut visibility = Vec::new();
    for &e in &mapping {
        let others = (0..visibility.len())
            .filter(|&o| !g.may_share(e, mapping[o]))
            .collect();
        visibility.push(others);
    }

    let mut best = Vec::new();
    let mut best_possible_improvement = vec![0];
    for first in 0..mapping.len() {
        let cx = Cx {
            mapping: &mapping[0..=first],
            visibility: &visibility[0..=first],
            best_possible_improvement: &best_possible_improvement,
        };

        let possible = visibility[first].clone();
        recur(cx, &mut vec![mapping[first]], &mut best, possible);
        best_possible_improvement.push(best.len());
    }

    best
}

#[derive(Clone, Copy)]
struct Cx<'x> {
    mapping: &'x [Entry],
    // Stores all entries in front of `index` not seen by index.
    visibility: &'x [TBitSet<usize>],
    // If we don't choose `index`, what's the best possible value
    // we can get.
    best_possible_improvement: &'x [usize],
}

fn recur(cx: Cx<'_>, chosen: &mut Vec<Entry>, best: &mut Vec<Entry>, mut possible: TBitSet<usize>) {
    if best.len() >= chosen.len() + possible.element_count() {
        return;
    }

    if let Some(first) = possible.iter().next_back() {
        if best.len() > chosen.len() + cx.best_possible_improvement[first] {
            return;
        }

        possible.remove(first);
        let f = cx.mapping[first];

        chosen.push(f);
        let new_possible = possible.intersection(&cx.visibility[first]);

        let ignore_check_without = new_possible == possible;
        recur(cx, chosen, best, new_possible);
        chosen.pop();

        if ignore_check_without {
            return;
        }
        // We don't choose `first`.
        if best.len() >= chosen.len() + cx.best_possible_improvement[first] {
            return;
        }

        recur(cx, chosen, best, possible);
    } else if chosen.len() > best.len() {
        best.clone_from(chosen);
    }
}
