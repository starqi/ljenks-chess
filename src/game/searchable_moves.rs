use std::collections::HashMap;
use super::coords::*;
use super::move_list::*;
use super::super::*;

/// (src, dest)
#[derive(Copy, Clone, PartialEq, Eq, Hash)]
struct SearchableMoveKey(FastCoord, FastCoord);

pub struct SearchableMoves {
    map: HashMap<SearchableMoveKey, MoveWithEval>
}

impl SearchableMoves {
    pub fn new() -> SearchableMoves {
        let map: HashMap<SearchableMoveKey, MoveWithEval> = HashMap::new();
        SearchableMoves { map }
    }

    pub fn reset(&mut self, curr_player: Player, move_list: &MoveList, start: usize, end_exclusive: usize) {

        self.map.clear();

        for i in start..end_exclusive {
            let m = &move_list.v()[i];

            match m.description() {
                MoveDescription::NormalMove(from, to) => {
                    self.map.insert(SearchableMoveKey(*from, *to), *m);
                }
                MoveDescription::Castle(castle_type) => {
                    match castle_type {
                        CastleType::Oo => {
                            for x in (CASTLE_UTILS.oo_sqs[curr_player as usize]).iter() {
                            }
                        },
                        CastleType::Ooo => {
                        }
                    }
                }
            }

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
