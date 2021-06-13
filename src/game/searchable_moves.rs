use std::collections::HashMap;
use super::coords::*;
use super::move_list::*;

/// (src, dest)
#[derive(Copy, Clone, PartialEq, Eq, Hash)]
struct SearchableMoveKey(Coord, Coord);

pub struct SearchableMoves {
    map: HashMap<SearchableMoveKey, MoveSnapshot>
}

impl SearchableMoves {
    pub fn new() -> SearchableMoves {
        let map: HashMap<SearchableMoveKey, MoveSnapshot> = HashMap::new();
        SearchableMoves { map }
    }

    pub fn reset(&mut self, move_list: &MoveList) {

        self.map.clear();

        for m in move_list.get_v() {
            if let Some(capture_dest) = m.get_dest_sq() {
                if let Some(capture_src) = m.get_src_sq() {
                    self.map.insert(SearchableMoveKey(capture_src.0, capture_dest.0), m.clone());
                    continue;
                }
            }

            if let MoveDescription::Oo = m.get_description() {
                let sqs = m.get_squares();
                self.map.insert(SearchableMoveKey(sqs[0].unwrap().0, sqs[3].unwrap().0), m.clone());
                self.map.insert(SearchableMoveKey(sqs[0].unwrap().0, sqs[2].unwrap().0), m.clone());
                self.map.insert(SearchableMoveKey(sqs[3].unwrap().0, sqs[0].unwrap().0), m.clone());
            } else if let MoveDescription::Ooo = m.get_description() {
                let sqs = m.get_squares();
                self.map.insert(SearchableMoveKey(sqs[0].unwrap().0, sqs[4].unwrap().0), m.clone());
                self.map.insert(SearchableMoveKey(sqs[4].unwrap().0, sqs[0].unwrap().0), m.clone());
                self.map.insert(SearchableMoveKey(sqs[4].unwrap().0, sqs[1].unwrap().0), m.clone());
                self.map.insert(SearchableMoveKey(sqs[4].unwrap().0, sqs[2].unwrap().0), m.clone());
            }
        }

        crate::console_log!("Searchable size - {}", self.map.len());
    }

    pub fn get_move(&self, from: Coord, to: Coord) -> Option<&MoveSnapshot> {
        match self.map.get(&SearchableMoveKey(from, to)) {
            Some(x) => Some(x),
            None => None
        }
    }
}
