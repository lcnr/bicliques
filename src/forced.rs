use crate::*;

pub(crate) fn forced_elements(g: &Bigraph) -> Vec<Entry> {
    let mut forced = Vec::new();
    for e in g.entries() {
        if forced
            .iter()
            .any(|f: &Entry| g.get(Entry(e.0, f.1)) && g.get(Entry(f.0, e.1)))
        {
            // part of an existing clique
        } else {
            forced.push(e);
        }
    }

    forced
}
