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
                    self.map.insert(SearchableMoveKey(*from, *to), m.clone());
                }
                MoveDescription::Castle(castle_type) => {
                    match castle_type {
                        CastleType::Oo => {
                            for (from, to) in (CASTLE_UTILS.oo_draggable_coords[curr_player as usize]).iter() {
                                self.map.insert(SearchableMoveKey(FastCoord::from_coord(from), FastCoord::from_coord(to)), m.clone());
                            }
                        },
                        CastleType::Ooo => {
                            for (from, to) in (CASTLE_UTILS.ooo_draggable_coords[curr_player as usize]).iter() {
                                self.map.insert(SearchableMoveKey(FastCoord::from_coord(from), FastCoord::from_coord(to)), m.clone());
                            }
                        }
                    }
                }
                _ => {
                }
            }
        }

        console_log!("Searchable size - {}", self.map.len());
    }

    pub fn get_move(&self, from: &Coord, to: &Coord) -> Option<&MoveWithEval> {
        match self.map.get(&SearchableMoveKey(FastCoord::from_coord(from), FastCoord::from_coord(to))) {
            Some(x) => Some(x),
            None => None
        }
    }
}
