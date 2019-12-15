use std::collections::HashMap;

type DocumentId = usize;
type WordPosition = usize;

pub type Entry = (DocumentId, WordPosition);

pub struct Index<T> {
    pub documents: Vec<T>,
    map: HashMap<String, Vec<Entry>>,
}

impl<T> Index<T> {
    pub fn new() -> Index<T> {
        Index {
            documents: Vec::<T>::new(),
            map: HashMap::new(),
        }
    }

    pub fn add(self: &mut Index<T>, source: &str, document: T) -> DocumentId {
        let id = self.documents.len();
        self.documents.push(document);

        for (idx, word) in source.split_whitespace().enumerate() {
            let entry: Entry = (id, idx);
            match self.map.get_mut(word) {
                Some(document_ids) => document_ids.push(entry),
                None => {
                    self.map.insert(String::from(word), vec![entry]);
                }
            };
        }

        id
    }

    pub fn find<'a>(self: &'a Index<T>, query: &str, matches: &mut Vec<Entry>) -> Option<()> {
        let mut words = query.split_whitespace();

        for entry in self.map.get(words.next()?)? {
            matches.push(*entry);
        }

        // TODO: optimize
        for (word_idx, word) in words.enumerate() {
            let mut to_remove = Vec::new();
            for (match_idx, (doc_id1, pos1)) in matches.iter().enumerate() {
                let mut keep = false;
                for (doc_id2, pos2) in self.map.get(word)? {
                    if *doc_id1 == *doc_id2 && *pos2 - *pos1 == word_idx + 1 {
                        keep = true;
                    }
                }
                if !keep {
                    to_remove.push(match_idx);
                }
            }

            to_remove.reverse();
            for idx in to_remove {
                matches.remove(idx);
            }
        }

        Some(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test<T>(index: &Index<T>, q: &str, expected: Vec<Entry>) {
        let mut matches = Vec::new();
        index.find(q, &mut matches);
        assert_eq!(matches, expected);
    }

    #[test]
    fn single_word() {
        let mut index = Index::<&str>::new();
        let d1 = index.add("one two three four", "d1");
        let d2 = index.add("two four six eight", "d2");
        let d3 = index.add("three six nine twelve", "d3");

        test(&index, "none", vec![]);

        test(&index, "two", vec![(d1, 1), (d2, 0)]);
        test(&index, "eight", vec![(d2, 3)]);
        test(&index, "six", vec![(d2, 2), (d3, 1)]);
    }

    #[test]
    fn multi_word() {
        let mut index = Index::<&str>::new();
        let d1 = index.add("one two three four", "d1");
        let d2 = index.add("two four six eight", "d2");
        let d3 = index.add("three six nine twelve", "d3");

        test(&index, "two three", vec![(d1, 1)]);
        test(&index, "two four", vec![(d2, 0)]);
        test(&index, "six nine twelve", vec![(d3, 1)]);
    }
}
