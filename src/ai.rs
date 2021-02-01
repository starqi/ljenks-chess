use rand::{thread_rng, Rng};
use super::board::{Board, MoveList, Piece, Square, CheckThreatTempBuffers, xy_to_file_rank_safe};

pub struct Ai<'a> {
    board: &'a mut Board,
    temp_board: Board,
    temp_buffers: CheckThreatTempBuffers,
    move_list: MoveList
}

impl Ai<'_> {
    pub fn new(board: &mut Board) -> Ai {
        Ai {
            board,
            temp_board: Board::new(),
            temp_buffers: CheckThreatTempBuffers::new(),
            move_list: MoveList::new()
        }
    }

    pub fn boom(&mut self) {

        self.temp_board.import_from(self.board);

        let mut rng = thread_rng();
        let current_player = self.board.get_player_with_turn();
        let current_player_state = self.board.get_player_state(current_player);

        let mut lowest_opponent_value: i32 = 9001;
        let mut lov_piece_loc: (u8, u8) = (0, 0);
        let mut lov_move_list_index: i8 = -1;

        for (p_x, p_y) in &current_player_state.piece_locs {

            let (file, rank) = xy_to_file_rank_safe(*p_x as i32, *p_y as i32).unwrap();
            self.board.get_moves(file, rank, &mut self.temp_buffers, &mut self.move_list).unwrap();

            let moves_v = self.move_list.get_moves();
            for i in 0..moves_v.len() {
                let revertable = self.temp_board.get_revertable_move(&self.move_list, i).unwrap();
                self.temp_board.make_move(&mut self.move_list, i).unwrap();

                let mut opponent_value: i32 = 0;
                for (x, y) in &self.temp_board.get_player_state(current_player.get_other_player()).piece_locs {
                    if let Ok(Square::Occupied(piece, _)) = self.temp_board.get_by_xy(*x, *y) {
                        opponent_value += match piece {
                            Piece::Queen => 9,
                            Piece::Pawn => 1,
                            Piece::Rook => 5,
                            Piece::Bishop => 3,
                            Piece::Knight => 3,
                            _ => 0
                        } * 10 + (4 - (*y as i32)).abs();
                    }
                }
                // FIXME Extract
                for (x, y) in &self.temp_board.get_player_state(current_player).piece_locs {
                    if let Ok(Square::Occupied(piece, _)) = self.temp_board.get_by_xy(*x, *y) {
                        opponent_value -= match piece {
                            Piece::Queen => 9,
                            Piece::Pawn => 1,
                            Piece::Rook => 5,
                            Piece::Bishop => 3,
                            Piece::Knight => 3,
                            _ => 0
                        } * 10 + (4 - (*y as i32)).abs();
                    }
                }
                opponent_value = ((opponent_value as f32) * rng.gen_range(0.8, 1.2)) as i32;

                if opponent_value < lowest_opponent_value {
                    lowest_opponent_value = opponent_value;
                    lov_piece_loc.0 = *p_x;
                    lov_piece_loc.1 = *p_y;
                    lov_move_list_index = i as i8;
                }

                self.temp_board.revert_move(&revertable).unwrap();
            }
        }

        if lov_move_list_index >= 0 {
            let (file, rank) = xy_to_file_rank_safe(lov_piece_loc.0 as i32, lov_piece_loc.1 as i32).unwrap();
            self.board.get_moves(file, rank, &mut self.temp_buffers, &mut self.move_list).unwrap();
            self.board.make_move(&mut self.move_list, lov_move_list_index as usize).unwrap();
        }

        println!("\n{}", self.board);
    }
}
