extern crate ngrammatic;

use self::ngrammatic::{CorpusBuilder, Pad, SearchResult};
use crate::HashMap;

pub trait RelevanceSearch<T> {
    fn sort_by_relevance(&mut self, by: T);
}

impl<T> RelevanceSearch<&str> for Vec<(String, T)> {
    fn sort_by_relevance(&mut self, by: &str) {
        let mut corpus = CorpusBuilder::new().arity(2).pad_full(Pad::Auto).finish();

        for (s, _) in self.iter() {
            corpus.add_text(s.as_str());
        }

        let mut all_items: HashMap<String, T> =
            std::mem::replace(self, vec![]).into_iter().collect();

        for SearchResult { text, .. } in corpus.search(by, 0f32) {
            if let Some(item) = all_items.remove(&text) {
                self.push((text, item));
            }
        }

        self.extend(all_items.into_iter());
    }
}
