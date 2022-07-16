use crate::*;

pub mod old {
    use crate::*;
    pub fn forced_elements(g: &Bigraph) -> Vec<Entry> {
        let mut best = Vec::new();
        let mapping: Vec<_> = g.entries().collect();
        let cx = Cx {
            g,
            mapping: &mapping,
        };
        recur(cx, &mut Vec::new(), &mut best, (0..mapping.len()).collect());
        best
    }
    
    #[derive(Clone, Copy)]
    struct Cx<'x> {
        g: &'x Bigraph,
        mapping: &'x [Entry],
    }
    
    fn recur(cx: Cx<'_>, chosen: &mut Vec<Entry>, best: &mut Vec<Entry>, mut possible: TBitSet<usize>) {
        if best.len() >= chosen.len() + possible.element_count() {
            return;
        }
    
        if let Some(first) = possible.iter().next() {
            // First recur while choosing `first`, then without.
            possible.remove(first);
            let f = cx.mapping[first];
    
            chosen.push(f);
            let new_possible = possible
                .iter()
                .filter(|&e| {
                    let e = cx.mapping[e];
                    !cx.g.may_share(e, f)
                })
                .collect();
            recur(cx, chosen, best, new_possible);
            chosen.pop();
    
            recur(cx, chosen, best, possible);
        } else {
            if chosen.len() > best.len() {
                best.clone_from(chosen);
            }
        }
    }
    

}

pub fn forced_elements(g: &Bigraph) -> Vec<Entry> {
    let mut best = Vec::new();
    let mapping: Vec<_> = g.entries().collect();
    let mut visibility = Vec::new();
    for &e in &mapping {
        let others = (0..mapping.len()).filter(|&o| !g.may_share(e, mapping[o])).collect();
        visibility.push(others);
    }
    let cx = Cx {
        mapping: &mapping,
        visibility: &visibility,
    };
    recur(cx, &mut Vec::new(), &mut best, (0..mapping.len()).collect());
    best
}

#[derive(Clone, Copy)]
struct Cx<'x> {
    mapping: &'x [Entry],
    // Stores all entries not seen by `index`.
    visibility: &'x [TBitSet<usize>],
}

fn recur(cx: Cx<'_>, chosen: &mut Vec<Entry>, best: &mut Vec<Entry>, mut possible: TBitSet<usize>) {
    if best.len() >= chosen.len() + possible.element_count() {
        return;
    }

    if let Some(first) = possible.iter().next() {
        // First recur while choosing `first`, then without.
        possible.remove(first);
        let f = cx.mapping[first];

        chosen.push(f);
        let new_possible = possible.intersection(&cx.visibility[first]);
        recur(cx, chosen, best, new_possible);
        chosen.pop();

        recur(cx, chosen, best, possible);
    } else {
        if chosen.len() > best.len() {
            best.clone_from(chosen);
        }
    }
}
