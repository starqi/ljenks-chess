// TODO How does this work as part of build process? i.e. Bindgen questions
import * as wasm from './node_modules/ljenks-chess';

function getBoardState() {
    const board = [];
    for (let y = 0; y < 8; y++) {
        for (let x = 0; x < 8; x++) {
            board.push(main.get_piece(x, y));
        }
    }
    return board;
}

async function loadWeights() {
    try {
        const resp = await fetch('nnue_model.safetensors');
        if (!resp.ok) throw new Error(`HTTP ${resp.status}`); // TODO IMMEDIATE Error handling in this func?
        const buffer = await resp.arrayBuffer();
        // Load into GLOBAL
        const ok = wasm.load_weights_safetensors(new Uint8Array(buffer));
        if (!ok) console.error('Failed to parse NNUE weights');
        // TODO IMMEDIATE Review error handling
    } catch (e) {
        console.error('Failed to load NNUE weights:', e);
    }
}

let main;
async function init() {
    await loadWeights();
    main = new wasm.Main();
    main.refresh_player_moves();
    postMessage({type: 'ready', board: getBoardState()});
}
init(); // Runs when worker created by main script

self.onmessage = (e) => {
    console.log('Worker received', e.data);
    if (e.data.type === 'make_ai_move') {
        const {depth, isAutoPlay} = e.data;
        const moveInfo = main.make_ai_move();
        if (!moveInfo) {
            postMessage({type: 'no_more_ai_moves'});
            return;
        }
        const board = getBoardState();
        const lastMoveStr = moveInfo.notation;
        const evaluation = moveInfo.score;
        if (!isAutoPlay) main.refresh_player_moves();
        postMessage({type: 'ai_move_done', isAutoPlay, board, lastMoveStr, evaluation, gameEndState: main.get_game_end_state()});
    } else if (e.data.type === 'make_human_move') {
        const {fromX, fromY, toX, toY, isAutoPlay} = e.data;
        const lastMoveStr = main.try_move(fromX, fromY, toX, toY);
        if (lastMoveStr) {
            const board = getBoardState();
            if (!isAutoPlay) main.refresh_player_moves();
            postMessage({type: 'human_move_done', isAutoPlay, board, lastMoveStr, gameEndState: main.get_game_end_state()});
        } else {
            postMessage({type: 'human_move_invalid'});
        }
    } else if (e.data.type === 'load_fen') {
        const {fen} = e.data;
        if (main.load_fen(fen)) {
            const board = getBoardState();
            const playerWithTurn = main.get_player_with_turn();
            const bestMoveInfo = main.evaluate();
            postMessage({type: 'fen_loaded', board, playerWithTurn, evaluation: bestMoveInfo.score, bestMoveStr: bestMoveInfo.notation});
        } else {
            postMessage({type: 'fen_invalid'});
        }
    }
};
