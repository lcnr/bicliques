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
    fn can_share(&self, a: Entry, b: Entry) -> bool {
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

    pub fn entries(&self) -> impl Iterator<Item = Entry> + '_ {
        self.entries
            .iter()
            .map(|index| self.entry_from_index(index))
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Biclique {
    left: TBitSet<u32>,
    right: TBitSet<u32>,
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BicliqueCover {
    elements: Box<[Biclique]>,
}

pub fn biclique_covers(g: &Bigraph, max_size: usize) -> impl Iterator<Item = BicliqueCover> + '_ {
    let forced_elements: Vec<Entry> = forced::forced_elements(g);

    covers::CoverIterator::new(g, max_size, forced_elements)
}
