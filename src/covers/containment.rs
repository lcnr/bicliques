use crate::*;

#[derive(Debug)]
struct Edge {
    data: Box<[Biclique]>,
    maximal: usize,
    empty: usize,
}

impl Edge {
    fn new(g: &Bigraph, mut data: Box<[Biclique]>) -> Edge {
        let mut maximal = 0;
        for i in 0..data.len() {
            if g.is_maximal(&data[i]) {
                data.swap(i, maximal);
                maximal += 1;
            }
        }

        let mut empty = data.len();
        for i in (maximal..data.len()).rev() {
            if data[i].is_empty() {
                empty -= 1;
                data.swap(i, empty);
            }
        }

        biclique_sort(&mut data[0..maximal]);

        Edge {
            data,
            maximal,
            empty,
        }
    }

    fn maximal(&self) -> &[Biclique] {
        &self.data[..self.maximal]
    }

    fn tail(&self) -> &[Biclique] {
        &self.data[self.maximal..self.empty]
    }

    fn non_empty(&self) -> &[Biclique] {
        &self.data[..self.empty]
    }

    fn empty(&self) -> usize {
        self.data.len() - self.empty
    }
}

pub(crate) struct Containment {
    entries: Vec<Edge>,
    layers: Vec<(usize, Box<[Biclique]>)>,
}

fn contains_reject(data: &[Biclique], edge: &Edge) -> bool {
    for c in edge.maximal() {
        if !data.iter().any(|q| q == c) {
            return false;
        }
    }

    if edge
        .tail()
        .iter()
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

fn contains_slow(data: &[Biclique], edge: &Edge) -> bool {
    let non_empty = edge.non_empty();
    let mut superset = vec![TBitSet::new(); non_empty.len()];
    for (i, c) in data.iter().enumerate() {
        if c.is_empty() {
            continue;
        }

        for (j, clique) in non_empty.iter().enumerate() {
            if c.contains_clique(clique) {
                superset[j].add(i);
            }
        }
    }

    solve_superset(superset)
}

/*
fn contains_stupid(data: &[Biclique], edge: &Edge) -> bool {
    let mut superset = vec![TBitSet::new(); edge.data.len()];
    for (j, clique) in edge.data.iter().enumerate() {
        for (i, c) in data.iter().enumerate() {
            if c.contains_clique(clique) {
                superset[j].add(i);
            }
        }
    }

    fn recurse(mut sup: Vec<TBitSet<usize>>) -> bool {
        match sup.pop() {
            Some(e) => {
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
*/

fn contains(data: &[Biclique], edge: &Edge) -> bool {
    contains_reject(data, edge) && contains_slow(data, edge)
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
        let clique = Edge::new(g, clique);
        debug_assert!(contains(&data, &clique));

        for child in self.entries.drain(start..) {
            debug_assert!(contains(&child.data, &clique));
        }

        self.entries.push(clique);
    }

    pub(crate) fn should_discard(&mut self, data: &[Biclique]) -> bool {
        let empty = data.iter().filter(|&c| c.is_empty()).count();
        for e in &self.entries {
            if empty > e.empty() {
                continue;
            }

            if contains(data, e) {
                return true;
            }
        }

        false
    }
}
