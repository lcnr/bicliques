use crate::*;

struct Entry {
    data: Box<[Biclique]>,
    maximal: usize,
}

impl Entry {
    fn new(g: &Bigraph, mut data: Box<[Biclique]>) -> Entry {
        let mut maximal = 0;
        for i in 0..data.len() {
            if g.is_maximal(&data[i]) {
                data.swap(i, maximal);
                maximal += 1;
            }
        }

        biclique_sort(&mut data[0..maximal]);

        Entry { data, maximal }
    }

    fn maximal(&self) -> impl Iterator<Item = &Biclique> + '_ {
        self.data.iter().take(self.maximal)
    }

    fn tail(&self) -> impl Iterator<Item = &Biclique> + '_ {
        self.data.iter().skip(self.maximal)
    }
}

pub(crate) struct Containment {
    entries: Vec<Entry>,
    layers: Vec<(usize, Box<[Biclique]>)>,
}

fn contains_reject(data: &[Biclique], entry: &Entry) -> bool {
    for c in entry.maximal() {
        if !data.iter().any(|q| q == c) {
            return false;
        }
    }

    if entry
        .tail()
        .any(|clique| data.iter().all(|c| !c.contains_clique(clique)))
    {
        return false;
    }

    true
}

fn solve_superset(mut superset: Vec<TBitSet<usize>>) -> bool {
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

fn contains_slow(data: &[Biclique], entry: &Entry) -> bool {
    let mut superset = Vec::new();
    for c in data.iter().filter(|&q| entry.maximal().all(|e| q != e)) {
        let mut sup = TBitSet::new();
        for (i, clique) in entry.tail().enumerate() {
            if c.contains_clique(clique) {
                sup.add(i);
            }
        }
        superset.push(sup);
    }

    solve_superset(superset)
}

fn contains(data: &[Biclique], entry: &Entry) -> bool {
    contains_reject(data, entry) && contains_slow(data, entry)
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

    pub(crate) fn finish_layer(&mut self, g: &Bigraph, data: Box<[Biclique]>) {
        let (start, clique) = self.layers.pop().unwrap();
        let clique = Entry::new(g, clique);
        debug_assert!(contains(&data, &clique));

        for child in self.entries.drain(start..) {
            debug_assert!(contains(&child.data, &clique));
        }

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
}
