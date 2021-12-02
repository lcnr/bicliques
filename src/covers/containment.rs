use crate::*;

pub(crate) struct Containment {
    entries: Vec<Box<[Biclique]>>,
    layers: Vec<(usize, Box<[Biclique]>)>,
}

fn contains(data: &[Biclique], other: &[Biclique]) -> bool {
    let mut superset = Vec::new();
    for c in data {
        let mut sup = TBitSet::new();
        for i in 0..other.len() {
            if c.contains_clique(&other[i]) {
                sup.add(i);
            }
        }
        superset.push(sup);
    }

    let mut changed = true;
    while changed {
        changed = false;
        let mut i = 0;
        while i < superset.len() {
            let mut items = superset[i].iter();
            let first = items.next();
            let second = items.next();
            match (first, second) {
                (None, None) => return false,
                (Some(fst), None) => {
                    changed = true;
                    superset.swap_remove(i);
                    for s in superset.iter_mut() {
                        s.remove(fst);
                    }
                }
                _ => i += 1,
            }
        }
    }

    fn recurse(mut sup: Vec<TBitSet<usize>>) -> bool {
        match sup.pop() {
            Some(e) => {
                if e.is_empty() {
                    return false;
                }

                for item in e {
                    let mut r = sup.clone();
                    for q in r.iter_mut() {
                        q.remove(item);
                    }

                    if recurse(r) {
                        return true;
                    }
                }

                false
            }
            None => true,
        }
    }

    recurse(superset)
}

impl Containment {
    pub(crate) fn init(init: &[Biclique]) -> Containment {
        let mut c = Containment {
            entries: Vec::new(),
            layers: vec![],
        };
        c.reinit(init);
        c
    }

    pub(crate) fn reinit(&mut self, init: &[Biclique]) {
        self.entries.clear();
        assert!(self.layers.is_empty());
        self.layers.push((0, init.to_owned().into_boxed_slice()));
    }

    /// Does there exists an existing partial biclique cover `X`
    /// for which you can construct a bijective function `sup[x] = y` where `x ⊆ y`.
    pub(crate) fn start_layer(&mut self, data: &[Biclique]) -> bool {
        if self.should_discard(data) {
            return false;
        }

        self.layers
            .push((self.entries.len(), data.to_owned().into_boxed_slice()));
        true
    }

    pub(crate) fn finish_layer(&mut self, data: Box<[Biclique]>) {
        let (start, clique) = self.layers.pop().unwrap();
        assert!(contains(&data, &clique));
        self.entries.truncate(start);
        self.entries.push(clique);
    }

    pub(crate) fn should_discard(&mut self, data: &[Biclique]) -> bool {
        for e in &self.entries {
            if contains(data, e) {
                return true;
            }
        }

        false
    }

    pub(crate) fn discard_layer(&mut self, data: Box<[Biclique]>) {
        let (start, clique) = self.layers.pop().unwrap();
        assert!(contains(&data, &clique));
        for child in self.entries.drain(start..) {
            assert!(contains(&child, &clique));
        }
        self.entries.push(clique);
    }
}
