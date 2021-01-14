// FIXME Bounds & string checks

#[derive(Copy, Clone, Debug)]
pub enum Piece {
    Pawn, Rook, Knight, Bishop, Queen, King
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum Player { 
    Black, White
}

#[derive(Copy, Clone, Debug)]
pub enum Square {
    Occupied(Piece, Player), Blank
}

#[derive(Debug)]
pub struct Board {
    arr: [Square; 64]
}

impl Board {
    pub fn new() -> Board {
        let mut board = Board {
            arr: [Square::Blank; 64]
        };

        board.set_main_row(1, Player::White);
        board.set_pawn_row(2, Player::White);

        board.set_main_row(8, Player::Black);
        board.set_pawn_row(7, Player::Black);

        board
    }

    pub fn set(&mut self, file: char, rank: u8, s: Square) {
        let (x, y) = Board::file_rank_to_xy(file, rank);
        self.set_by_xy(x, y, s);
    }

    pub fn get(&self, file: char, rank: u8) -> Square {
        let (x, y) = Board::file_rank_to_xy(file, rank);
        self.get_by_xy(x, y)
    }

    pub fn get_local_legal_moves(&self, file: char, rank: u8, result: &mut Vec<(char, u8)>) {
        result.clear();

        let (_x, _y) = Board::file_rank_to_xy(file, rank);
        let (x, y) = (_x as i8, _y as i8);

        if let Square::Occupied(piece, player) = self.get_by_xy(_x, _y) {
            match piece {
                Piece::Pawn => {
                },
                Piece::Rook => {
                },
                Piece::Knight => {
                    self.push_candidate(x - 1, y + 2, player, result);
                    self.push_candidate(x - 1, y - 2, player, result);

                    self.push_candidate(x - 2, y + 1, player, result);
                    self.push_candidate(x - 2, y - 1, player, result);

                    self.push_candidate(x + 2, y + 1, player, result);
                    self.push_candidate(x + 2, y - 1, player, result);

                    self.push_candidate(x + 1, y + 2, player, result);
                    self.push_candidate(x + 1, y - 2, player, result);
                },
                Piece::Bishop => {
                },
                Piece::Queen => {
                },
                Piece::King => {
                },
                _ => {

                }
            }
        }
    }

    fn set_pawn_row(&mut self, rank: u8, player: Player) {
        for i in 0..8 {
            self.set_by_xy(i, 8 - rank as usize, Square::Occupied(Piece::Pawn, player));
        }
    }

    fn set_main_row(&mut self, rank: u8, player: Player) {
        self.set('a', rank, Square::Occupied(Piece::Rook, player));
        self.set('b', rank, Square::Occupied(Piece::Knight, player));
        self.set('c', rank, Square::Occupied(Piece::Bishop, player));
        self.set('d', rank, Square::Occupied(Piece::Queen, player));
        self.set('e', rank, Square::Occupied(Piece::King, player));
        self.set('f', rank, Square::Occupied(Piece::Bishop, player));
        self.set('g', rank, Square::Occupied(Piece::Knight, player));
        self.set('h', rank, Square::Occupied(Piece::Rook, player));
    }

    fn get_by_xy(&self, x: usize, y: usize) -> Square {
        return self.arr[y * 8 + x];
    }

    fn set_by_xy(&mut self, x: usize, y: usize, s: Square) {
        self.arr[y * 8 + x] = s;
    }

    //////////////////////////////////////////////////

    fn push_candidate(&self, x: i8, y: i8, player_owner: Player, result: &mut Vec<(char, u8)>) {
        if x >= 0 && x <= 7 && y >= 0 && y <= 7 {
            let should_push = match self.get_by_xy(x as usize, y as usize) {
                Square::Occupied(piece, player) => {
                    player != player_owner
                },
                Square::Blank => {
                    true
                }
            };
            if should_push {
                result.push(Board::xy_to_file_rank(x as usize, y as usize));
            }
        }
    }

    fn xy_to_file_rank(x: usize, y: usize) -> (char, u8) {
        (std::char::from_u32(x as u32 + ('a' as u32)).unwrap(), 8 - (y as u8))
    }

    fn file_rank_to_xy(file: char, rank: u8) -> (usize, usize) {
        let x = file as u32 - 'a' as u32;
        let y = 8 - rank;
        (x as usize, y as usize)
    }
}
