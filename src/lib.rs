use std::ops::ControlFlow;
use tindex::TBitSet;

mod covers;
pub mod forced;
pub mod old;

#[cfg(test)]
mod tests;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Edge(pub u32, pub u32);

#[derive(Debug)]
pub struct Bigraph {
    left: u32,
    edge_x_offset: u8,
    right: u32,
    entries: TBitSet<usize>,
}

impl Bigraph {
    pub fn new(left: u32, right: u32) -> Bigraph {
        Bigraph {
            left,
            edge_x_offset: right.next_power_of_two().trailing_zeros() as u8,
            right,
            entries: TBitSet::new(),
        }
    }

    pub fn left(&self) -> u32 {
        self.left
    }

    pub fn right(&self) -> u32 {
        self.right
    }

    #[inline(always)]
    fn edge_index(&self, Edge(x, y): Edge) -> usize {
        let x = (x as usize) << self.edge_x_offset;
        x + y as usize
    }

    #[inline(always)]
    fn edge_from_index(&self, index: usize) -> Edge {
        let x = index >> self.edge_x_offset;
        let y = index & !(!0 << self.edge_x_offset);
        Edge(x as u32, y as u32)
    }

    #[inline(always)]
    pub fn get(&self, e: Edge) -> bool {
        self.entries.get(self.edge_index(e))
    }

    pub fn add(&mut self, e: Edge) {
        self.entries.add(self.edge_index(e))
    }

    #[inline(always)]
    fn may_share(&self, a: Edge, b: Edge) -> bool {
        self.get(Edge(a.0, b.1)) && self.get(Edge(b.0, a.1))
    }

    fn may_add(&self, clique: &Biclique, e: Edge) -> bool {
        for x in clique.left.iter() {
            if !self.get(Edge(x, e.1)) {
                return false;
            }
        }

        for y in clique.right.iter() {
            if !self.get(Edge(e.0, y)) {
                return false;
            }
        }

        true
    }

    pub fn is_maximal(&self, clique: &Biclique) -> bool {
        for x in 0..self.left {
            if !clique.left.get(x) {
                if clique.right.iter().all(|y| self.get(Edge(x, y))) {
                    return false;
                }
            }
        }

        for y in 0..self.right {
            if !clique.right.get(y) {
                if clique.left.iter().all(|x| self.get(Edge(x, y))) {
                    return false;
                }
            }
        }

        true
    }

    pub fn is_maximal_cover(&self, cover: &BicliqueCover) -> bool {
        cover.elements.iter().all(|clique| self.is_maximal(clique))
    }

    pub fn entries(&self) -> impl Iterator<Item = Edge> + Clone + '_ {
        self.entries
            .iter()
            .map(|index| self.edge_from_index(index))
    }

    pub fn left_entries(&self, x: u32) -> impl Iterator<Item = Edge> + '_ {
        (0..self.right)
            .map(move |y| Edge(x, y))
            .filter(|&e| self.get(e))
    }

    pub fn right_entries(&self, y: u32) -> impl Iterator<Item = Edge> + '_ {
        (0..self.left)
            .map(move |x| Edge(x, y))
            .filter(|&e| self.get(e))
    }
}

impl<const L: usize, const R: usize> From<[[bool; R]; L]> for Bigraph {
    fn from(arr: [[bool; R]; L]) -> Bigraph {
        let mut g = Bigraph::new(L as u32, R as u32);
        for (x, row) in arr.iter().enumerate() {
            for (y, &set) in row.iter().enumerate() {
                if set {
                    g.add(Edge(x as u32, y as u32));
                }
            }
        }
        g
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Biclique {
    pub left: TBitSet<u32>,
    pub right: TBitSet<u32>,
}

fn biclique_sort(bicliques: &mut [Biclique]) {
    use std::cmp::Ordering;
    let bitset_ord = |a: &TBitSet<u32>, b: &TBitSet<u32>| {
        let mut a_iter = a.iter().rev();
        let mut b_iter = b.iter().rev();
        loop {
            match (a_iter.next(), b_iter.next()) {
                (None, None) => return Ordering::Equal,
                (a, b) => match a.cmp(&b) {
                    Ordering::Equal => (),
                    ord => return ord,
                },
            }
        }
    };

    bicliques.sort_by(|a, b| {
        bitset_ord(&a.left, &b.left)
            .then_with(|| bitset_ord(&a.right, &b.right))
            .reverse()
    });
}

impl Biclique {
    fn empty() -> Biclique {
        Biclique {
            left: TBitSet::new(),
            right: TBitSet::new(),
        }
    }

    pub fn left(&self) -> impl Iterator<Item = u32> + '_ {
        self.left.iter()
    }

    pub fn right(&self) -> impl Iterator<Item = u32> + '_ {
        self.right.iter()
    }

    fn is_empty(&self) -> bool {
        self.left.is_empty() && self.right.is_empty()
    }

    fn contains(&self, edge: Edge) -> bool {
        self.left.get(edge.0) && self.right.get(edge.1)
    }

    fn contains_clique(&self, other: &Biclique) -> bool {
        self.left.contains(&other.left) && self.right.contains(&other.right)
    }

    fn eq(&self, other: &Biclique) -> bool {
        self.left == other.left && self.right == other.right
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct BicliqueCover {
    elements: Box<[Biclique]>,
}

impl BicliqueCover {
    fn new(g: &Bigraph, elements: Box<[Biclique]>) -> Self {
        let mut this = BicliqueCover { elements };
        this.consistent(g);
        this.canonicalize();
        this
    }

    fn consistent(&self, g: &Bigraph) {
        cfg_if::cfg_if! {
            if #[cfg(debug_assertions)] {
                for x in 0..g.left {
                    for y in 0..g.right {
                        assert_eq!(
                            g.get(Edge(x, y)),
                            self.elements.iter().any(|c| c.contains(Edge(x, y)))
                        );
                    }
                }
            } else {
                let _ = g;
            }
        }
    }

    fn canonicalize(&mut self) {
        biclique_sort(&mut self.elements)
    }

    pub fn cliques(&self) -> &[Biclique] {
        &self.elements
    }

    pub fn print(&self, g: &Bigraph) -> String {
        let mut s = String::new();
        for c in self.elements.iter() {
            for i in 0..g.left() {
                if c.left.get(i) {
                    s.push('1');
                } else {
                    s.push('0');
                }
            }

            s.push('|');

            for i in 0..g.right() {
                if c.right.get(i) {
                    s.push('1');
                } else {
                    s.push('0');
                }
            }

            s.push(' ');
        }

        s.pop();
        s
    }
}

pub fn biclique_covers<T, F: FnMut(BicliqueCover) -> ControlFlow<T>>(
    g: &Bigraph,
    max_size: usize,
    f: F,
) -> ControlFlow<T> {
    let forced_elements: Vec<Edge> = forced::forced_elements(g);

    covers::iterate(g, max_size, forced_elements, f)
}
