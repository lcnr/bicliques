use crate::*;

pub fn forced_elements_old(g: &Bigraph) -> Vec<Entry> {
    let mut best = Vec::new();
    recur_old(g, &mut Vec::new(), &mut best, g.entries.clone());
    best
}

fn recur_old(
    g: &Bigraph,
    chosen: &mut Vec<Entry>,
    best: &mut Vec<Entry>,
    mut possible: TBitSet<usize>,
) {
    if let Some(first) = possible.iter().next() {
        // First recur_old while choosing `first`, then without.
        possible.remove(first);
        let f = g.entry_from_index(first);

        chosen.push(f);
        let new_possible = possible
            .iter()
            .filter(|&e| {
                let e = g.entry_from_index(e);
                !g.may_share(e, f)
            })
            .collect();
        recur_old(g, chosen, best, new_possible);
        chosen.pop();

        if best.len() < chosen.len() + possible.element_count() {
            recur_old(g, chosen, best, possible);
        }
    } else {
        if chosen.len() > best.len() {
            best.clone_from(chosen);
        }
    }
}

pub fn forced_elements(g: &Bigraph) -> Vec<Entry> {
    let mut best = Vec::new();
    recur(g, &mut Vec::new(), &mut best, g.entries.clone());
    best
}

fn recur(
    g: &Bigraph,
    chosen: &mut Vec<Entry>,
    best: &mut Vec<Entry>,
    mut possible: TBitSet<usize>,
) {
    if best.len() >= chosen.len() + possible.element_count() {
        return;
    }

    if let Some(first) = possible.iter().next() {
        // First recur while choosing `first`, then without.
        possible.remove(first);
        let f = g.entry_from_index(first);

        chosen.push(f);
        let new_possible = possible
            .iter()
            .filter(|&e| {
                let e = g.entry_from_index(e);
                !g.may_share(e, f)
            })
            .collect();
        recur(g, chosen, best, new_possible);
        chosen.pop();

        recur(g, chosen, best, possible);
    } else {
        if chosen.len() > best.len() {
            best.clone_from(chosen);
        }
    }
}
