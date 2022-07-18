use crate::*;

pub fn forced_elements(g: &Bigraph) -> Vec<Edge> {
    let mut mapping: Vec<_> = g.entries().collect();

    let mut guaranteed = optimal_forced_elements(&mapping);
    mapping.retain(|&e| guaranteed.iter().all(|&o| !g.may_share(e, o)));

    let dominated_entries = dominated_entries(g, &mapping);
    for e in dominated_entries {
        mapping.retain(|&o| o != e);
    }

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

    guaranteed.extend(best);
    guaranteed
}

/// Given a bigraph like the one below, including `X`
/// as a forced element is always optimal as it only
/// blocks other entries in its row.
///
/// Other entries in its row may block some entries in
/// other rows as well.
///
/// ```plain
/// Xx_xx
/// ___xx
/// ```
///
/// The usefulness of this decreases with the size and fullness
/// of the bigraph.
fn optimal_forced_elements(mapping: &[Edge]) -> Vec<Edge> {
    let mut guaranteed: Vec<Edge> = Vec::new();
    for (i, &e) in mapping.iter().enumerate() {
        let mut x_ok = true;
        let mut y_ok = true;
        for o in mapping[..i].iter().chain(&mapping[i + 1..]) {
            if e.0 == o.0 {
                x_ok = false;
            }
            if e.1 == o.1 {
                y_ok = false;
            }
        }

        if (y_ok || x_ok) && guaranteed.iter().all(|o| e.0 != o.0 && e.1 != o.1) {
            guaranteed.push(e);
        }
    }

    guaranteed
}

fn dominated_entries(g: &Bigraph, mapping: &[Edge]) -> Vec<Edge> {
    for &Edge(x, y) in mapping {
        // For all entries in row
    }
    for x in 0..g.left() {
        for y in 0..g.right() {
            if !g.get(Edge(x, y)) {
                continue;
            }


        }
    }
    Vec::new()
}

#[derive(Clone, Copy)]
struct Cx<'x> {
    mapping: &'x [Edge],
    // Stores all entries in front of `index` not seen by index.
    visibility: &'x [TBitSet<usize>],
    // If we don't choose `index`, what's the best possible value
    // we can get.
    best_possible_improvement: &'x [usize],
}

fn recur(cx: Cx<'_>, chosen: &mut Vec<Edge>, best: &mut Vec<Edge>, mut possible: TBitSet<usize>) {
    if best.len() >= chosen.len() + possible.element_count() {
        return;
    }

    if let Some(first) = possible.iter().next_back() {
        // Choosing `first`.
        if best.len() > chosen.len() + cx.best_possible_improvement[first] {
            return;
        }

        let f = cx.mapping[first];

        chosen.push(f);
        let new_possible = possible.intersection(&cx.visibility[first]);

        let ignore_check_without = new_possible == possible;
        recur(cx, chosen, best, new_possible);
        chosen.pop();

        // We don't choose `first`.
        if ignore_check_without {
            return;
        }

        if best.len() >= chosen.len() + cx.best_possible_improvement[first] {
            return;
        }

        possible.remove(first);
        recur(cx, chosen, best, possible);
    } else if chosen.len() > best.len() {
        best.clone_from(chosen);
    }
}
