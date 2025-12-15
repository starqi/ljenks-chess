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
        main.make_ai_move(depth);
        const board = getBoardState();
        const lastMoveStr = main.get_last_move_notation();
        if (!isAutoPlay) main.refresh_player_moves();
        postMessage({type: 'ai_move_done', isAutoPlay, board, lastMoveStr});
    } else if (e.data.type === 'make_human_move') {
        const {fromX, fromY, toX, toY, isAutoPlay} = e.data;
        if (main.try_move(fromX, fromY, toX, toY)) {
            const board = getBoardState();
            const lastMoveStr = main.get_last_move_notation();
            if (!isAutoPlay) main.refresh_player_moves();
            postMessage({type: 'human_move_done', isAutoPlay, board, lastMoveStr});
        } else {
            // TODO IMMEDIATE Handle
            postMessage({type: 'move_invalid'});
        }
    }
};

main.refresh_player_moves();
postMessage({type: 'ready', board: getBoardState()});
