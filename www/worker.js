import * as wasm from './node_modules/ljenks-chess';

let main = wasm.Main.new();

function getBoardState() {
    const board = [];
    for (let y = 0; y < 8; y++) {
        for (let x = 0; x < 8; x++) {
            board.push(main.get_piece(x, y));
        }
    }
    return board;
}

self.onmessage = (e) => {
    console.log('Worker received', e.data);
    if (e.data.type === 'make_ai_move') {
        const {depth, isAutoPlay} = e.data;
        if (!main.make_ai_move()) {
            postMessage({type: 'move_invalid'});
            return;
        }
        const board = getBoardState();
        const lastMoveStr = main.get_last_move_notation();
        const evaluation = main.get_last_ai_evaluation();
        if (!isAutoPlay) main.refresh_player_moves();
        postMessage({type: 'ai_move_done', isAutoPlay, board, lastMoveStr, evaluation, gameEndState: main.get_game_end_state()});
    } else if (e.data.type === 'make_human_move') {
        const {fromX, fromY, toX, toY, isAutoPlay} = e.data;
        if (main.try_move(fromX, fromY, toX, toY)) {
            const board = getBoardState();
            const lastMoveStr = main.get_last_move_notation();
            if (!isAutoPlay) main.refresh_player_moves();
            postMessage({type: 'human_move_done', isAutoPlay, board, lastMoveStr, gameEndState: main.get_game_end_state()});
        } else {
            postMessage({type: 'move_invalid'});
        }
    } else if (e.data.type === 'load_fen') {
        const {fen} = e.data;
        if (main.load_fen(fen)) {
            const board = getBoardState();
            const playerWithTurn = main.get_player_with_turn();
            postMessage({type: 'fen_loaded', board, playerWithTurn});
        } else {
            postMessage({type: 'fen_invalid'});
        }
    }
};

main.refresh_player_moves();
postMessage({type: 'ready', board: getBoardState()});
