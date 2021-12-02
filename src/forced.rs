use crate::*;

fn entry_forced(g: &Bigraph, forced: &[Entry], e: Entry) -> bool {
    forced
        .iter()
        .any(|f: &Entry| g.get(Entry(e.0, f.1)) && g.get(Entry(f.0, e.1)))
}

fn element_cost(g: &Bigraph, forced: &[Entry], e: Entry) -> u32 {
    let mut cost = 0;
    for q in g.entries() {
        if !entry_forced(g, forced, q) && g.may_share(q, e) {
            cost += 1;
        }
    }

    cost
}

pub(crate) fn forced_elements(g: &Bigraph) -> Vec<Entry> {
    let mut forced = Vec::new();
    loop {
        let mut best = None;
        let mut value = u32::MAX;
        for e in g.entries() {
            if !entry_forced(g, &forced, e) {
                let cost = element_cost(g, &forced, e);
                if cost < value {
                    value = cost;
                    best = Some(e);
                }
            }
        }

        if let Some(best) = best {
            forced.push(best);
        } else {
            break forced;
        }
    }
}
