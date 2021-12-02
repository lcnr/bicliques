use std::ops::ControlFlow;
use tindex::TBitSet;

mod covers;
mod forced;

#[derive(Clone, Copy)]
pub struct Entry(pub u32, pub u32);

#[derive(Debug)]
pub struct Bigraph {
    left: u32,
    entry_x_offset: u8,
    right: u32,
    entries: TBitSet<usize>,
}

impl Bigraph {
    pub fn new(left: u32, right: u32) -> Bigraph {
        Bigraph {
            left,
            entry_x_offset: right.next_power_of_two().trailing_zeros() as u8,
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
    fn entry_index(&self, Entry(x, y): Entry) -> usize {
        let x = (x as usize) << self.entry_x_offset;
        x + y as usize
    }

    #[inline(always)]
    fn entry_from_index(&self, index: usize) -> Entry {
        let x = index >> self.entry_x_offset;
        let y = index & !(!0 << self.entry_x_offset);
        Entry(x as u32, y as u32)
    }

    pub fn get(&self, e: Entry) -> bool {
        self.entries.get(self.entry_index(e))
    }

    pub fn add(&mut self, e: Entry) {
        self.entries.add(self.entry_index(e))
    }

    #[inline(always)]
    fn may_share(&self, a: Entry, b: Entry) -> bool {
        self.get(Entry(a.0, b.1)) && self.get(Entry(b.0, a.1))
    }

    fn may_add(&self, clique: &Biclique, e: Entry) -> bool {
        for x in clique.left.iter() {
            if !self.get(Entry(x, e.1)) {
                return false;
            }
        }

        for y in clique.right.iter() {
            if !self.get(Entry(e.0, y)) {
                return false;
            }
        }

        true
    }

    pub fn is_maximal(&self, clique: &Biclique) -> bool {
        for x in 0..self.left {
            if !clique.left.get(x) {
                if clique.right.iter().all(|y| self.get(Entry(x, y))) {
                    return false;
                }
            }
        }

        for y in 0..self.right {
            if !clique.right.get(y) {
                if clique.left.iter().all(|x| self.get(Entry(x, y))) {
                    return false;
                }
            }
        }

        true
    }

    pub fn is_maximal_cover(&self, cover: &BicliqueCover) -> bool {
        cover.elements.iter().all(|clique| self.is_maximal(clique))
    }

    pub fn entries(&self) -> impl Iterator<Item = Entry> + '_ {
        self.entries
            .iter()
            .map(|index| self.entry_from_index(index))
    }

    pub fn left_entries(&self, x: u32) -> impl Iterator<Item = Entry> + '_ {
        (0..self.right)
            .map(move |y| Entry(x, y))
            .filter(|&e| self.get(e))
    }

    pub fn right_entries(&self, y: u32) -> impl Iterator<Item = Entry> + '_ {
        (0..self.left)
            .map(move |x| Entry(x, y))
            .filter(|&e| self.get(e))
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Biclique {
    pub left: TBitSet<u32>,
    pub right: TBitSet<u32>,
}

impl Biclique {
    fn empty() -> Biclique {
        Biclique {
            left: TBitSet::new(),
            right: TBitSet::new(),
        }
    }

    fn contains(&self, entry: Entry) -> bool {
        self.left.get(entry.0) && self.right.get(entry.1)
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
                            g.get(Entry(x, y)),
                            self.elements.iter().any(|c| c.contains(Entry(x, y)))
                        );
                    }
                }
            } else {
                let _ = g;
            }
        }
    }

    fn canonicalize(&mut self) {
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

        self.elements.sort_by(|a, b| {
            bitset_ord(&a.left, &b.left)
                .then_with(|| bitset_ord(&a.right, &b.right))
                .reverse()
        });
    }

    pub fn cliques(&self) -> impl Iterator<Item = &Biclique> + '_ {
        self.elements.iter()
    }
}

pub fn biclique_covers<F: FnMut(BicliqueCover) -> ControlFlow<()>>(
    g: &Bigraph,
    max_size: usize,
    f: F,
) {
    let forced_elements: Vec<Entry> = forced::forced_elements(g);

    covers::iterate(g, max_size, forced_elements, f);
}
