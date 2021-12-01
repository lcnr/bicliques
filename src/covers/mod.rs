use crate::*;

mod containment;

use containment::Containment;

#[derive(Debug, Clone)]
struct Layer {
    /// The part which is currently forced.
    bicliques: Box<[Biclique]>,
    data: TBitSet<usize>,
}

const IN_BICLIQUE: usize = 0;
const POSSIBILITY_OFFSET: usize = IN_BICLIQUE + 1;

#[derive(Copy, Clone)]
struct DataIndex(usize);

impl DataIndex {
    fn in_biclique(self) -> usize {
        self.0 + IN_BICLIQUE
    }

    fn may_add(self, i: usize) -> usize {
        self.0 + POSSIBILITY_OFFSET + i
    }
}

impl Layer {
    fn index(g: &Bigraph, k: usize, e: Entry) -> DataIndex {
        DataIndex(g.entry_index(e) * (k + POSSIBILITY_OFFSET))
    }

    fn in_biclique(i: DataIndex) -> usize {
        i.0 + IN_BICLIQUE
    }

    fn initial(g: &Bigraph, k: usize, forced: &[Entry]) -> Layer {
        let mut bicliques: Vec<Biclique> = forced
            .iter()
            .map(|&Entry(x, y)| Biclique {
                left: [x].into_iter().collect(),
                right: [y].into_iter().collect(),
            })
            .collect();
        bicliques.resize(k, Biclique::empty());

        // WARNING: `data` is still inconsistent here,
        // we have to be careful about which methods we use.
        let mut layer = Layer {
            bicliques: bicliques.into_boxed_slice(),
            data: TBitSet::new(),
        };

        for e in g.entries() {
            let index = Layer::index(g, k, e);
            for i in 0..k {
                if layer.bicliques[i].contains(e) {
                    layer.data.add(index.in_biclique());
                } else if g.may_add(&layer.bicliques[i], e) {
                    layer.data.add(index.may_add(i));
                };
            }
        }

        layer.consistent(g);

        layer
    }

    fn cliques(&self) -> impl Iterator<Item = usize> {
        0..self.bicliques.len()
    }

    fn covers(&self, g: &Bigraph) -> bool {
        for e in g.entries() {
            if self.bicliques.iter().all(|c| !c.contains(e)) {
                return false;
            }
        }

        true
    }

    fn consistent(&self, g: &Bigraph) {
        for x in 0..g.left {
            for y in 0..g.right {
                let entry = Entry(x, y);
                let index = Layer::index(g, self.bicliques.len(), entry);
                if g.get(entry) {
                    assert_eq!(
                        self.data.get(index.in_biclique()),
                        self.bicliques.iter().any(|c| c.contains(entry))
                    );

                    for c in self.cliques() {
                        if self.data.get(index.may_add(c)) {
                            assert!(g.may_add(&self.bicliques[c], entry));
                            assert!(!self.bicliques[c].contains(entry));
                        }
                    }
                } else {
                    assert!(!self.data.get(index.in_biclique()));
                    for c in self.cliques() {
                        assert!(!self.data.get(index.may_add(c)));
                    }
                }
            }
        }
    }

    fn add_left(&mut self, g: &Bigraph, c: usize, x: u32) {
        for y in self.bicliques[c].right.iter() {
            let index = Layer::index(g, self.bicliques.len(), Entry(x, y));
            self.data.add(index.in_biclique());
            self.data.remove(index.may_add(c));
        }

        'outer: for y in 0..g.right {
            if g.get(Entry(x, y)) {
                continue 'outer;
            }

            for x in self.bicliques[c].left.iter() {
                if !g.get(Entry(x, y)) {
                    continue 'outer;
                }
            }

            for x in 0..g.left {
                if g.get(Entry(x, y)) {
                    let index = Layer::index(g, self.bicliques.len(), Entry(x, y));
                    self.data.remove(index.may_add(c));
                }
            }
        }
        self.bicliques[c].left.add(x);

        self.consistent(g)
    }

    fn add_right(&mut self, g: &Bigraph, c: usize, y: u32) {
        for x in self.bicliques[c].left.iter() {
            let index = Layer::index(g, self.bicliques.len(), Entry(x, y));
            self.data.add(index.in_biclique());
            self.data.remove(index.may_add(c));
        }

        'outer: for x in 0..g.left {
            if g.get(Entry(x, y)) {
                continue 'outer;
            }

            for y in self.bicliques[c].right.iter() {
                if !g.get(Entry(x, y)) {
                    continue 'outer;
                }
            }

            for y in 0..g.right {
                if g.get(Entry(x, y)) {
                    let index = Layer::index(g, self.bicliques.len(), Entry(x, y));
                    self.data.remove(index.may_add(c));
                }
            }
        }

        self.bicliques[c].right.add(y);

        self.consistent(g)
    }

    fn add_entry(&mut self, g: &Bigraph, c: usize, e: Entry) {
        if !self.bicliques[c].left.get(e.0) {
            self.add_left(g, c, e.0);
        }

        if !self.bicliques[c].right.get(e.1) {
            self.add_right(g, c, e.1);
        }

        self.consistent(g)
    }

    fn may_add_iter(&self, index: DataIndex) -> impl Iterator<Item = usize> + '_ {
        (0..self.bicliques.len()).filter(move |&i| self.data.get(index.may_add(i)))
    }

    fn forced_updates(&mut self, g: &Bigraph) -> Result<bool, ()> {
        let mut changed = true;
        while changed {
            let mut found_ambig = false;
            changed = false;
            'entries: for e in g.entries() {
                let index = Layer::index(g, self.bicliques.len(), e);
                if self.data.get(index.in_biclique()) {
                    continue;
                }

                let mut entry = None;
                for i in self.cliques() {
                    if self.data.get(index.may_add(i)) {
                        match entry {
                            None => entry = Some(i),
                            Some(_) => {
                                found_ambig = true;
                                continue 'entries;
                            }
                        }
                    }
                }

                if let Some(c) = entry {
                    changed = true;
                    self.add_entry(g, c, e);
                } else {
                    return Err(());
                }
            }

            if !found_ambig && changed {
                // New correct biclique.
                return Ok(true);
            }
        }

        // Ambiguity
        Ok(false)
    }

    /// Guesses an entry, removing it from `self` and returning
    /// a new layer with the chosen entry.
    fn guess_entry(&mut self, g: &Bigraph) -> Option<Layer> {
        // TODO: be clever here
        for e in g.entries() {
            let index = Layer::index(g, self.bicliques.len(), e);
            'cliques: for c in self.cliques() {
                if self.data.get(index.may_add(c)) {
                    let mut new_layer = self.clone();
                    new_layer.add_entry(g, c, e);
                    for i in new_layer.cliques() {
                        if i != c && new_layer.bicliques[c].eq(&new_layer.bicliques[i]) {
                            continue 'cliques;
                        }
                    }

                    self.data.remove(index.may_add(c));
                    self.consistent(g);
                    return Some(new_layer);
                }
            }
        }

        None
    }
}

pub struct CoverIterator<'a> {
    g: &'a Bigraph,
    k: usize,
    max_size: usize,
    forced: Vec<Entry>,
    containment: Containment,
    stack: Vec<Layer>,
}

impl<'a> CoverIterator<'a> {
    pub(crate) fn new(g: &'a Bigraph, max_size: usize, forced: Vec<Entry>) -> CoverIterator<'a> {
        let k = forced.len();
        let initial = Layer::initial(g, k, &forced);

        CoverIterator {
            g,
            k,
            max_size,
            forced,
            containment: Containment::init(&initial.bicliques),
            stack: if k > max_size { vec![] } else { vec![initial] },
        }
    }

    fn finish_layer(&mut self) -> Option<BicliqueCover> {
        self.containment
            .finish_layer(self.stack.pop().unwrap().bicliques);
        self.next()
    }

    fn discard_layer(&mut self) -> Option<BicliqueCover> {
        self.containment
            .discard_layer(self.stack.pop().unwrap().bicliques);
        self.next()
    }
}

impl<'a> Iterator for CoverIterator<'a> {
    type Item = BicliqueCover;

    fn next(&mut self) -> Option<BicliqueCover> {
        if let Some(layer) = self.stack.last_mut() {
            match layer.forced_updates(self.g) {
                Ok(true) => Some(BicliqueCover {
                    elements: layer.bicliques.clone(),
                }),
                Ok(false) => {
                    while let Some(new_layer) = layer.guess_entry(self.g) {
                        if self.containment.start_layer(&new_layer.bicliques) {
                            if new_layer.covers(self.g) {
                                let elements = new_layer.bicliques.clone();
                                self.stack.push(new_layer);
                                return Some(BicliqueCover { elements });
                            } else {
                                self.stack.push(new_layer);
                                return self.next();
                            }
                        }
                    }

                    self.finish_layer()
                }
                Err(()) => self.finish_layer(),
            }
        } else if self.k < self.max_size {
            self.k += 1;
            let init = Layer::initial(self.g, self.k, &self.forced);
            self.containment.reinit(&init.bicliques);
            self.stack.push(init);
            self.next()
        } else {
            None
        }
    }
}
