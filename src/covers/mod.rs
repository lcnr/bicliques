use crate::*;
use std::mem;

mod containment;

use containment::Containment;

#[derive(Debug, Clone)]
struct Layer {
    /// The part which is currently forced.
    bicliques: Box<[Biclique]>,
    data: TBitSet<usize>,
    changed: TBitSet<usize>,
}

// per edge offsets
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
    fn index(g: &Bigraph, k: usize, e: Edge) -> DataIndex {
        DataIndex(g.edge_index(e) * (k + POSSIBILITY_OFFSET))
    }

    fn initial(g: &Bigraph, k: usize, forced: &[Edge]) -> Layer {
        let mut bicliques: Vec<Biclique> = forced
            .iter()
            .map(|&Edge(x, y)| Biclique {
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
            changed: (0..k).collect(),
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
        g.entries().all(|e| {
            self.data
                .get(Layer::index(g, self.bicliques.len(), e).in_biclique())
        })
    }

    fn consistent(&self, g: &Bigraph) {
        cfg_if::cfg_if! {
            if #[cfg(debug_assertions)] {
                for x in 0..g.left {
                    for y in 0..g.right {
                        let edge = Edge(x, y);
                        let index = Layer::index(g, self.bicliques.len(), edge);
                        if g.get(edge) {
                            assert_eq!(
                                self.data.get(index.in_biclique()),
                                self.bicliques.iter().any(|c| c.contains(edge))
                            );

                            for c in self.cliques() {
                                if self.data.get(index.may_add(c)) {
                                    assert!(g.may_add(&self.bicliques[c], edge));
                                    assert!(!self.bicliques[c].contains(edge));
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
            } else {
                let _ = g;
            }
        }
    }

    fn add_left(&mut self, g: &Bigraph, c: usize, x: u32) {
        for y in self.bicliques[c].right.iter() {
            let index = Layer::index(g, self.bicliques.len(), Edge(x, y));
            self.data.add(index.in_biclique());
            self.data.remove(index.may_add(c));
        }

        'outer: for y in 0..g.right {
            if g.get(Edge(x, y)) {
                continue 'outer;
            }

            for x in self.bicliques[c].left.iter() {
                if !g.get(Edge(x, y)) {
                    continue 'outer;
                }
            }

            for x in 0..g.left {
                if g.get(Edge(x, y)) {
                    let index = Layer::index(g, self.bicliques.len(), Edge(x, y));
                    self.data.remove(index.may_add(c));
                }
            }
        }

        self.bicliques[c].left.add(x);
        self.changed.add(c);

        self.consistent(g)
    }

    fn add_right(&mut self, g: &Bigraph, c: usize, y: u32) {
        for x in self.bicliques[c].left.iter() {
            let index = Layer::index(g, self.bicliques.len(), Edge(x, y));
            self.data.add(index.in_biclique());
            self.data.remove(index.may_add(c));
        }

        'outer: for x in 0..g.left {
            if g.get(Edge(x, y)) {
                continue 'outer;
            }

            for y in self.bicliques[c].right.iter() {
                if !g.get(Edge(x, y)) {
                    continue 'outer;
                }
            }

            for y in 0..g.right {
                if g.get(Edge(x, y)) {
                    let index = Layer::index(g, self.bicliques.len(), Edge(x, y));
                    self.data.remove(index.may_add(c));
                }
            }
        }

        self.bicliques[c].right.add(y);
        self.changed.add(c);

        self.consistent(g)
    }

    fn add_edge(&mut self, g: &Bigraph, c: usize, e: Edge) {
        if !self.bicliques[c].left.get(e.0) {
            self.add_left(g, c, e.0);
        }

        if !self.bicliques[c].right.get(e.1) {
            self.add_right(g, c, e.1);
        }

        self.consistent(g)
    }

    fn forced_updates(&mut self, g: &Bigraph) -> Result<(), ()> {
        let mut changed = true;
        while changed {
            changed = false;
            'entries: for e in g.entries() {
                let index = Layer::index(g, self.bicliques.len(), e);
                if self.data.get(index.in_biclique()) {
                    continue;
                }

                let mut edge = None;
                for i in self.cliques() {
                    if self.data.get(index.may_add(i)) {
                        match edge {
                            None => edge = Some(i),
                            Some(_) => {
                                continue 'entries;
                            }
                        }
                    }
                }

                if let Some(c) = edge {
                    changed = true;
                    self.add_edge(g, c, e);
                } else {
                    return Err(());
                }
            }
        }

        // Ambiguity
        Ok(())
    }

    /// Guesses an edge, removing it from `self` and returning
    /// a new layer with the chosen edge.
    fn guess_edge(&mut self, g: &Bigraph) -> Option<Layer> {
        for max_choices in 2..self.bicliques.len() {
            for e in g.entries() {
                let index = Layer::index(g, self.bicliques.len(), e);
                let num_choices = self
                    .cliques()
                    .filter(|&c| self.data.get(index.may_add(c)))
                    .count();
                if num_choices > max_choices {
                    continue;
                }

                'cliques: for c in self.cliques() {
                    if self.data.get(index.may_add(c)) {
                        let mut new_layer = self.clone();
                        new_layer.add_edge(g, c, e);
                        for i in new_layer.cliques() {
                            if i != c && new_layer.bicliques[c].eq(&new_layer.bicliques[i]) {
                                continue 'cliques;
                            }
                        }

                        let prev_cliques = &self.bicliques[c];
                        if prev_cliques.left.get(e.0) {
                            for x in 0..g.left {
                                let index = Layer::index(g, self.bicliques.len(), Edge(x, e.1));
                                self.data.remove(index.may_add(c));
                            }
                        } else if prev_cliques.right.get(e.1) {
                            for y in 0..g.right {
                                let index = Layer::index(g, self.bicliques.len(), Edge(e.0, y));
                                self.data.remove(index.may_add(c));
                            }
                        } else if prev_cliques.left.is_empty() && prev_cliques.right.is_empty() {
                            self.data.remove(index.may_add(c));
                        } else {
                            continue 'cliques;
                        }

                        self.consistent(g);
                        return Some(new_layer);
                    }
                }
            }
        }

        None
    }
}

fn iterate_sat<T, F: FnMut(BicliqueCover) -> ControlFlow<T>>(
    g: &Bigraph,
    containment: &mut Containment,
    mut layer: Layer,
    f: &mut F,
) -> ControlFlow<T> {
    while let Some(mut new_layer) = layer.guess_edge(g) {
        match restrict_layer(g, &mut new_layer) {
            Ok(()) => (),
            Err(()) => continue,
        }

        if containment.start_layer(&new_layer.bicliques) {
            iterate_sat(g, containment, new_layer, f)?;
        }
    }

    containment.finish_layer(g, layer.bicliques.clone());
    f(BicliqueCover::new(g, layer.bicliques.clone()))
}

fn left_maximal(g: &Bigraph, layer: &mut Layer, c: usize) {
    let mut maximal: TBitSet<u32> = (0..g.right).collect();
    for x in layer.bicliques[c].left.iter() {
        for y in 0..g.right {
            if !g.get(Edge(x, y)) {
                maximal.remove(y)
            }
        }
    }

    'left: for x in 0..g.left {
        for y in maximal.iter() {
            if !g.get(Edge(x, y)) {
                continue 'left;
            }
        }

        layer.add_left(g, c, x);
    }
}

fn right_maximal(g: &Bigraph, layer: &mut Layer, c: usize) {
    let mut maximal: TBitSet<u32> = (0..g.left).collect();
    for y in layer.bicliques[c].right.iter() {
        for x in 0..g.left {
            if !g.get(Edge(x, y)) {
                maximal.remove(x)
            }
        }
    }

    'right: for y in 0..g.right {
        for x in maximal.iter() {
            if !g.get(Edge(x, y)) {
                continue 'right;
            }
        }

        layer.add_right(g, c, y);
    }
}

fn restrict_layer(g: &Bigraph, layer: &mut Layer) -> Result<(), ()> {
    for c in mem::take(&mut layer.changed) {
        right_maximal(g, layer, c);
        left_maximal(g, layer, c);
    }
    layer.changed.clear();
    layer.forced_updates(g)
}

pub(crate) fn iterate<T, F: FnMut(BicliqueCover) -> ControlFlow<T>>(
    g: &Bigraph,
    max_size: usize,
    forced: Vec<Edge>,
    mut f: F,
) -> ControlFlow<T> {
    for k in forced.len()..=max_size {
        let layer = Layer::initial(g, k, &forced);
        let mut containment = Containment::init(&layer.bicliques);
        let mut stack = vec![layer];
        'cliques: while let Some(layer) = stack.last_mut() {
            match layer.forced_updates(g) {
                Ok(()) => (),
                Err(()) => {
                    containment.finish_layer(g, stack.pop().unwrap().bicliques);
                    continue 'cliques;
                }
            }

            match restrict_layer(g, layer) {
                Ok(()) => (),
                Err(()) => {
                    containment.finish_layer(g, stack.pop().unwrap().bicliques);
                    continue 'cliques;
                }
            }

            if containment.should_discard(&layer.bicliques) {
                containment.finish_layer(g, stack.pop().unwrap().bicliques);
                continue 'cliques;
            }

            if layer.covers(g) {
                iterate_sat(g, &mut containment, stack.pop().unwrap(), &mut f)?;
            } else {
                while let Some(new_layer) = layer.guess_edge(g) {
                    if containment.start_layer(&new_layer.bicliques) {
                        stack.push(new_layer);
                        continue 'cliques;
                    }
                }

                containment.finish_layer(g, stack.pop().unwrap().bicliques);
            }
        }
    }

    ControlFlow::Continue(())
}
