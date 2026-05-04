export function getBoardState(main) {
    const board = [];
    for (let y = 0; y < 8; y++) {
        for (let x = 0; x < 8; x++) {
            board.push(main.get_piece(x, y));
        }
    }
    return board;
}

export function initWorker(main) {
    main.refresh_player_moves();
    self.onmessage = (e) => {
        console.log('Worker received', e.data);
        if (e.data.type === 'make_ai_move') {
            const {depth, isAutoPlay} = e.data;
            const moveInfo = main.make_ai_move();
            if (!moveInfo) {
                postMessage({type: 'no_more_ai_moves'}); // Note this is an error state, see index.js
                return;
            }
            const board = getBoardState(main);
            const lastMoveStr = moveInfo.notation;
            const evaluation = moveInfo.score;
            if (!isAutoPlay) main.refresh_player_moves();
            postMessage({type: 'ai_move_done', isAutoPlay, board, lastMoveStr, evaluation, gameEndState: main.get_game_end_state()});
        } else if (e.data.type === 'make_human_move') {
            const {fromX, fromY, toX, toY, isAutoPlay} = e.data;
            const lastMoveStr = main.try_move(fromX, fromY, toX, toY);
            if (lastMoveStr) {
                const board = getBoardState(main);
                if (!isAutoPlay) main.refresh_player_moves();
                postMessage({type: 'human_move_done', isAutoPlay, board, lastMoveStr, gameEndState: main.get_game_end_state()});
            } else {
                postMessage({type: 'human_move_invalid'});
            }
        } else if (e.data.type === 'load_fen') {
            const {fen} = e.data;
            if (main.load_fen(fen)) {
                const board = getBoardState(main);
                const playerWithTurn = main.get_player_with_turn();
                const bestMoveInfo = main.evaluate();
                postMessage({type: 'fen_loaded', board, playerWithTurn, evaluation: bestMoveInfo.score, bestMoveStr: bestMoveInfo.notation});
            } else {
                postMessage({type: 'fen_invalid'});
            }
        }
    };
    postMessage({type: 'ready', board: getBoardState(main)});
}
