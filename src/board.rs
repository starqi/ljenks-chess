#[derive(Copy, Clone, Debug, PartialEq)]
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
    arr: [Square; 64],
    player_with_turn: Player,
    revision: u32
}

#[derive(Copy, Clone, Debug)]
pub enum Error {
    Unknown
}

pub struct MoveList {
    v: Vec<(char, u8)>,
    source: Option<(char, u8)>,
    revision: u32
}

impl MoveList {
    pub fn new() -> MoveList {
        MoveList { 
            v: Vec::new(),
            source: Option::None,
            revision: 0 
        }
    }

    pub fn get_moves(&self) -> &Vec<(char, u8)> {
        &self.v
    }
}

impl Board {
    pub fn new() -> Board {
        let mut board = Board {
            arr: [Square::Blank; 64],
            player_with_turn: Player::White,
            revision: 0
        };

        board.set_main_row(1, Player::White);
        //board.set_pawn_row(2, Player::White);

        board.set_main_row(8, Player::Black);
        //board.set_pawn_row(7, Player::Black);

        board
    }

    pub fn set(&mut self, file: char, rank: u8, s: Square) -> Option<Error> {
        match Board::file_rank_to_xy_safe(file, rank) {
            Result::Ok((x, y)) => {
                self.set_by_xy(x, y, s);
                Option::None
            },
            Result::Err(e) => Option::Some(e)
        }
    }

    pub fn get(&self, file: char, rank: u8) -> Result<Square, Error> {
        match Board::file_rank_to_xy_safe(file, rank) {
            Result::Ok((x, y)) => Result::Ok(self.get_by_xy(x, y)),
            Result::Err(e) => Result::Err(e)
        }
    }

    pub fn make_move(&mut self, moves: &mut MoveList, index: usize) -> Option<Error> {
        match moves.source {
            Option::None => {
                eprintln!("Move list is no longer valid");
                Option::Some(Error::Unknown)
            },
            Option::Some((source_file, source_rank)) => {
                if let Option::Some((file, rank)) = moves.v.get(index) {
                    match self.get(source_file, source_rank) {
                        Result::Ok(source_square) => {
                            self.set(*file, *rank, source_square);
                            self.set(source_file, source_rank, Square::Blank);
                            moves.source = Option::None;
                            self.player_with_turn = match self.player_with_turn {
                                Player::Black => Player::White,
                                Player::White => Player::Black,
                            };
                            Option::None
                        },
                        Result::Err(e) => {
                            Option::Some(e)
                        }
                    }
                } else {
                    eprintln!("Move list index out of bounds {} / {}", index, moves.v.len());
                    Option::Some(Error::Unknown)
                }
            }
        }
    }

    pub fn get_legal_moves(&self, file: char, rank: u8, result: &mut MoveList) -> Option<Error> {
        result.v.clear();
        result.revision = self.revision;
        result.source = Option::Some((file, rank));

        let (x_us, y_us) = match Board::file_rank_to_xy_safe(file, rank) {
            Result::Ok((x_us, y_us)) => (x_us, y_us),
            Result::Err(e) => {
                return Option::Some(e);
            }
        };
        let (x, y) = (x_us as i8, y_us as i8);

        if let Square::Occupied(piece, player) = self.get_by_xy(x_us, y_us) {
            match piece {
                Piece::Pawn => {
                },
                Piece::Rook => {
                    self.push_rook_candidates(x, y, player, &mut result.v);
                },
                Piece::Knight => {

                    self.push_candidate(x - 1, y + 2, player, &mut result.v);
                    self.push_candidate(x - 1, y - 2, player, &mut result.v);

                    self.push_candidate(x - 2, y + 1, player, &mut result.v);
                    self.push_candidate(x - 2, y - 1, player, &mut result.v);

                    self.push_candidate(x + 2, y + 1, player, &mut result.v);
                    self.push_candidate(x + 2, y - 1, player, &mut result.v);

                    self.push_candidate(x + 1, y + 2, player, &mut result.v);
                    self.push_candidate(x + 1, y - 2, player, &mut result.v);
                },
                Piece::Bishop => {
                    self.push_bishop_candidates(x, y, player, &mut result.v);
                },
                Piece::Queen => {
                    self.push_rook_candidates(x, y, player, &mut result.v);
                    self.push_bishop_candidates(x, y, player, &mut result.v);
                },
                Piece::King => {
                    for i in -1..=1 {
                        for j in -1..=1 {
                            if i == 0 && j == 0 {
                                continue;
                            }
                            self.push_candidate(x + i, y + j, player, &mut result.v);
                        }
                    }
                }
            }
        }

        Option::None
    }

    fn push_rook_candidates(&self, x: i8, y: i8, player: Player, result: &mut Vec<(char, u8)>) {
        for _i in 1..=x {
            let i = x - _i;
            if self.push_candidate(i, y, player, result) { break; }
        }
        for i in x + 1..=7 {
            if self.push_candidate(i, y, player, result) { break; }
        }
        for _i in 1..=y {
            let i = y - _i;
            if self.push_candidate(x, i, player, result) { break; }
        }
        for i in y + 1..=7 {
            if self.push_candidate(x, i, player, result) { break; }
        }
    }

    fn push_bishop_candidates(&self, x: i8, y: i8, player: Player, result: &mut Vec<(char, u8)>) {
        for i in 1..=x {
            if self.push_candidate(x - i, y - i, player, result) { break; }
        }
        for i in 1..=x {
            if self.push_candidate(x - i, y + i, player, result) { break; }
        }
        for i in 1..=8 - (x + 1) {
            if self.push_candidate(x + i, y - i, player, result) { break; }
        }
        for i in 1..=8 - (x + 1) {
            if self.push_candidate(x + i, y + i, player, result) { break; }
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
        self.revision += 1;
    }

    // Checks are for blind arithmetic
    // Returns whether to terminate the direction (if applicable)
    fn push_candidate(&self, x: i8, y: i8, player_owner: Player, result: &mut Vec<(char, u8)>) -> bool {
        if player_owner == self.player_with_turn && x >= 0 && x <= 7 && y >= 0 && y <= 7 {
            let (moveable, terminate) = match self.get_by_xy(x as usize, y as usize) {
                Square::Occupied(piece, player) => {
                    (player != player_owner && piece != Piece::King, true)
                },
                Square::Blank => {
                    (true, false)
                }
            };
            if moveable {
                result.push(Board::xy_to_file_rank(x as usize, y as usize));
            }
            return terminate;
        } else {
            return true;
        }
    }

    //////////////////////////////////////////////////

    fn xy_to_file_rank(x: usize, y: usize) -> (char, u8) {
        (std::char::from_u32(x as u32 + ('a' as u32)).unwrap(), 8 - (y as u8))
    }

    fn file_rank_to_xy(file: char, rank: u8) -> (usize, usize) {
        let x = file as u32 - 'a' as u32;
        let y = 8 - rank;
        (x as usize, y as usize)
    }

    // Checks are for public interface
    fn file_rank_to_xy_safe(file: char, rank: u8) -> Result<(usize, usize), Error> {
        if rank < 1 || rank > 8 {
            eprintln!("Rank out of bounds - {}", rank);
            return Result::Err(Error::Unknown);
        }
        let file_u32 = file as u32;
        if file_u32 < 'a' as u32 || file_u32 > 'h' as u32 {
            eprintln!("File out of bounds - {}", file);
            return Result::Err(Error::Unknown);
        }
        return Result::Ok(Board::file_rank_to_xy(file, rank));
    }
}
