import bb from './assets/bb.png';
import bw from './assets/bw.png';
import kb from './assets/kb.png';
import kw from './assets/kw.png';
import nb from './assets/nb.png';
import nw from './assets/nw.png';
import pb from './assets/pb.png';
import pw from './assets/pw.png';
import qb from './assets/qb.png';
import qw from './assets/qw.png';
import rb from './assets/rb.png';
import rw from './assets/rw.png';

const imageUrls = {
    bb, bw, kb, kw, nb, nw, pb, pw, qb, qw, rb, rw
};

class Application {

    static TEMP_DEPTH = 7;

    constructor() {
        // Rust code is thought of from white perspective, then conversion is made in JS if playing as black.
        this.isWhiteCameraPosition = true;

        // [Locking]
        // Lock specifically the ability for human to drag pieces (currently, the only action possible except for the side buttons)
        // for EVERY request submission to worker, so worker thinking is always a frozen board.
        // Clicking on the side buttons however will reset the whole worker and the whole screen immediately and cancel all worker operations.
        // Therefore, when the worker comes back, then unlock. 
        // If the worker coming back auto-triggers another worker session (auto-play) then don't unlock.
        this.boardPieceDragLock = false;

        this.draggedImage = null;
        this.draggedSqX = 0;
        this.draggedSqY = 0;

        this.moveHistory = [];
        this.moveNumber = 1;

        this.SQUARE_LENGTH = (0.75 * Math.min(
            window.innerWidth - document.getElementById('move-list').getBoundingClientRect().width,
            window.innerHeight - document.getElementById('title').getBoundingClientRect().height
        ) / 8) >>> 0;

        // Pawn = 0, Rook, Knight, Bishop, Queen, King
        this.numToLetter = [
            'p', 'r', 'n', 'b', 'q', 'k'
        ];

        this.board = document.getElementById('board');
        this.board.addEventListener('mousedown', this.onBoardMouseDown.bind(this));
        this.board.addEventListener('mousemove', this.onBoardMouseMove.bind(this));
        this.board.addEventListener('mouseup', this.onBoardMouseUp.bind(this));
        this.board.addEventListener('touchstart', this.onTouchStart.bind(this));
        this.board.addEventListener('touchmove', this.onTouchMove.bind(this));
        this.board.addEventListener('touchend', this.onTouchEnd.bind(this));

        this.evalText = document.getElementById('eval-text');
        this.evalBar = document.getElementById('eval-bar');

        document.getElementById('self-play-btn').addEventListener('click', () => {
            this.onSelfPlayButtonClick();
        });
        document.getElementById('play-btn').addEventListener('click', () => {
            this.onPlayButtonClick();
        });
        document.getElementById('load-btn').addEventListener('click', () => {
            this.onLoadButtonClick();
        });

        this.dragged = /** @type {HTMLImageElement} */ (document.getElementById('dragged'));
        this.dragged.width = this.SQUARE_LENGTH;
        this.dragged.height = this.SQUARE_LENGTH;

        this.squareImages = [];
        this.lastBoardState = new Array(64);

        for (let i = 0; i < 8; ++i) {
            const rowElement = document.createElement('div');
            const imageRow = [];
            const delta = i % 2 === 0 ? 0 : 1;

            for (let i = 0; i < 8; ++i) {
                const square = document.createElement('span');
                square.style.width = this.SQUARE_LENGTH + 'px';
                square.style.height = this.SQUARE_LENGTH + 'px';
                square.style.display = 'inline-block';
                square.style.backgroundColor = (i + delta) % 2 === 0 ? '#eeeeee' : '#915355';
                square.dataset.backgroundColor = square.style.backgroundColor;

                const image = new Image();
                image.width = this.SQUARE_LENGTH;
                image.height = this.SQUARE_LENGTH;
                image.style.visibility = 'hidden';
                image.src = '';

                square.append(image);
                rowElement.append(square);
                imageRow.push(image);
            }

            this.board.append(rowElement);
            this.squareImages.push(imageRow);
        }

        this.worker = null;
        
        document.getElementById('fen-ok').addEventListener('click', () => {
            this.onFenPopupOk();
        });
        document.getElementById('fen-cancel').addEventListener('click', () => {
            this.closeFenPopup();
        });
        document.getElementById('fen-overlay').addEventListener('click', () => {
            this.closeFenPopup();
        });
        document.addEventListener('keydown', (e) => {
            if (document.getElementById('fen-popup').style.display === 'block') {
                if (e.key === 'Enter') {
                    this.onFenPopupOk();
                } else if (e.key === 'Escape') {
                    this.closeFenPopup();
                }
            }
            if (document.getElementById('game-over-popup').style.display === 'block') {
                if (e.key === 'Escape' || e.key === 'Enter') {
                    this.closeGameOverPopup();
                }
            }
        });
        
        document.getElementById('game-over-close').addEventListener('click', () => {
            this.closeGameOverPopup();
        });
        document.getElementById('game-over-overlay').addEventListener('click', () => {
            this.closeGameOverPopup();
        });
        
        this.onPlayButtonClick();
    }

    showGameOverPopup(state) {
        const title = document.getElementById('game-over-title');
        const body = document.getElementById('game-over-body');
        
        let message = "";
        let titleText = "Game Over";
        
        switch(state) {
            case 0: // WhiteWin
                titleText = "Checkmate!";
                message = "White wins!";
                break;
            case 1: // BlackWin
                titleText = "Checkmate!";
                message = "Black wins!";
                break;
            case 2: // Stalemate
                titleText = "Draw";
                message = "Stalemate";
                break;
            case 3: // Repetition
                titleText = "Draw";
                message = "Repetition or 50-move rule";
                break;
        }
        
        title.textContent = titleText;
        body.textContent = message;
        document.getElementById('game-over-overlay').style.display = 'block';
        document.getElementById('game-over-popup').style.display = 'block';
    }

    closeGameOverPopup() {
        document.getElementById('game-over-overlay').style.display = 'none';
        document.getElementById('game-over-popup').style.display = 'none';
    }

    //////////////////////////////////////////////////
    // Web worker interface

    reset(onReady, isWhiteCameraPosition) {
        if (isWhiteCameraPosition === undefined || isWhiteCameraPosition === null) isWhiteCameraPosition = true;

        // Creating worker before termination dodges Firefox "bug" causing massive NPS slow down,
        // hypothesis: internals not liking terminate() immediately followed by making a new worker due to some sort of compiled code clean up.   
        // Using shared thread array for quick termination -> hoops to jump through regarding security and providing "require-corp" headers.
        console.log('Creating new worker before terminate');
        const replacementWorker = new Worker(new URL('worker.js', import.meta.url));
        if (this.worker) {
            this.worker.terminate();
            this.worker = null;
        }
        this.boardPieceDragLock = true;
        this.closeGameOverPopup();

        this.worker = replacementWorker;
        this.worker.onerror = (e) => {
            console.error('Worker error: ', e);
        };
        this.worker.onmessage = (e) => {
            console.log('Worker response', e.data);

            if (e.data.type === 'ready') {
                this.isWhiteCameraPosition = isWhiteCameraPosition;
                // Twice to get rid of board "diffs" between old and new boards
                this.refreshBoardFromWasmData(e.data.board);
                this.refreshBoardFromWasmData(e.data.board);

                this.moveHistory = [];
                this.moveNumber = 1;
                this.redrawMoveList();
                this.updateEvaluation(null);

                this.boardPieceDragLock = false;

                onReady(); // AFTER everything is done
            } else if (e.data.type === 'ai_move_done') {
                this.addLastMoveToHistory(e.data.lastMoveStr);
                this.refreshBoardFromWasmData(e.data.board);
                if (e.data.evaluation !== undefined) {
                    this.updateEvaluation(e.data.evaluation);
                }

                if (e.data.gameEndState !== undefined && e.data.gameEndState !== null) {
                    this.boardPieceDragLock = true;
                    this.showGameOverPopup(e.data.gameEndState);
                } else if (e.data.isAutoPlay) {
                    this.scheduleSelfPlayChain();
                } else {
                    this.boardPieceDragLock = false;
                }
            } else if (e.data.type === 'human_move_done') {
                this.addLastMoveToHistory(e.data.lastMoveStr);
                this.showDraggedOriginalSquare(); // A bit ugly: Need to be before refreshBoardFromWasmData() due to visibility CSS needing to be invisible as the final state
                this.refreshBoardFromWasmData(e.data.board);
                
                if (e.data.gameEndState !== undefined && e.data.gameEndState !== null) {
                    this.boardPieceDragLock = true;
                    this.showGameOverPopup(e.data.gameEndState);
                } else if (e.data.isAutoPlay) {
                    this.makeAiMoveAsync(Application.TEMP_DEPTH, false);
                } else {
                    this.boardPieceDragLock = false;
                }
            } else if (e.data.type === 'fen_loaded') {
                // Set camera position based on player with turn (0=White, 1=Black)
                this.isWhiteCameraPosition = e.data.playerWithTurn === 0;
                // Twice to get rid of board "diffs" between old and new boards
                this.refreshBoardFromWasmData(e.data.board);
                this.refreshBoardFromWasmData(e.data.board);
                this.moveHistory = [];
                this.moveNumber = 1;
                this.redrawMoveList();
                this.updateEvaluation(e.data.evaluation);
                this.boardPieceDragLock = false;
            } else if (e.data.type === 'fen_invalid') {
                alert('Invalid FEN string');
                this.boardPieceDragLock = false;
            } else if (e.data.type === 'human_move_invalid') {
                this.showDraggedOriginalSquare();
                this.boardPieceDragLock = false;
            } else if (e.data.type === 'no_more_ai_moves') {
                console.error('AI has no moves but game end was not detected!');
            } else {
                alert('Unexpected error: Unknown type ' + e.data.type);
            }
        };
    }

    makeAiMoveAsync(depth, isAutoPlay) {
        this.boardPieceDragLock = true;
        this.worker.postMessage({type: 'make_ai_move', depth, isAutoPlay});
    }

    makeHumanMoveAsync(fromX, fromY, toX, toY, isAutoPlay) {
        this.boardPieceDragLock = true;
        this.worker.postMessage({type: 'make_human_move', fromX, fromY, toX, toY, isAutoPlay});
    }

    scheduleSelfPlayChain() {
        this.makeAiMoveAsync(Application.TEMP_DEPTH, true);
    }

    //////////////////////////////////////////////////
    // Side buttons

    buttonFlipper = false; // ONLY used by play and self-play buttons 

    onPlayButtonClick() {
        this.buttonFlipper = !this.buttonFlipper;
        this.reset(() => {
            if (!this.isWhiteCameraPosition) {
                this.makeAiMoveAsync(Application.TEMP_DEPTH, false);
            }
        }, this.buttonFlipper);
    }

    onSelfPlayButtonClick() {
        this.buttonFlipper = !this.buttonFlipper;
        this.reset(() => {
            this.scheduleSelfPlayChain();
        }, this.buttonFlipper);
    }

    onLoadButtonClick() {
        this.showFenPopup();
    }

    showFenPopup() {
        document.getElementById('fen-popup').style.display = 'block';
        document.getElementById('fen-overlay').style.display = 'block';
        /** @type {HTMLInputElement} */ (document.getElementById('fen-input')).value = '';
        document.getElementById('fen-input').focus();
    }

    closeFenPopup() {
        document.getElementById('fen-popup').style.display = 'none';
        document.getElementById('fen-overlay').style.display = 'none';
    }

    onFenPopupOk() {
        const fen = /** @type {HTMLInputElement} */ (document.getElementById('fen-input')).value.trim();
        if (fen) {
            this.reset(() => {
                this.boardPieceDragLock = true;
                this.worker.postMessage({type: 'load_fen', fen});
                this.closeFenPopup();
            });
        } else {
            this.closeFenPopup();
        }
    }

    //////////////////////////////////////////////////
    // Generic piece move (touch, mouse) code

    onGenericDragStart(clientX, clientY) {

        if (this.draggedImage !== null) {
            // Normal behaviour = draggedImage becomes null on mouse up
            // But if any shenanigans with mouse/touch up not being called, then clean up the invisible piece
            this.draggedImage.style.visibility = 'visible';
        }

        const sqCoords = this.getSquareCoordsFromClientCoords(clientX, clientY);

        const row = this.squareImages[sqCoords.y];
        if (row === undefined) return;
        const image = row[sqCoords.x];
        if (image === undefined || image.style.visibility === 'hidden') return;

        image.style.visibility = 'hidden';
        this.draggedImage = image;
        this.dragged.src = image.src;
        this.dragged.style.visibility = 'visible';
        this.draggedSqY = sqCoords.y;
        this.draggedSqX = sqCoords.x;
        this.trySyncDragged(clientX, clientY);
    }

    trySyncDragged(clientX, clientY) {
        const boardCoords = this.getBoardCoordsFromClientCoords(clientX, clientY);
        if (this.draggedImage !== null) {
            this.dragged.style.left = (boardCoords.x - this.SQUARE_LENGTH / 2.0).toString();
            this.dragged.style.top = (boardCoords.y - this.SQUARE_LENGTH / 2.0).toString();
        }
    }

    onGenericDragEnd(clientX, clientY) {
        if (this.draggedImage === null) return; // Shouldn't happen

        this.dragged.style.visibility = 'hidden';
        if (this.boardPieceDragLock) { // Snap dragged piece back if locked, otherwise submit move and wait
            this.showDraggedOriginalSquare();
            return;
        }

        const sqCoords = this.getSquareCoordsFromClientCoords(clientX, clientY);
        if (this.isWhiteCameraPosition) {
            this.makeHumanMoveAsync(
                this.draggedSqX,
                this.draggedSqY,
                sqCoords.x,
                sqCoords.y,
                true
            );
        } else {
            this.makeHumanMoveAsync(
                7 - this.draggedSqX,
                7 - this.draggedSqY,
                7 - sqCoords.x,
                7 - sqCoords.y,
                true
            );
        }
    }

    showDraggedOriginalSquare() {
        if (this.draggedImage) {
            this.draggedImage.style.visibility = 'visible';
            this.draggedImage = null;
        }
    }

    //////////////////////////////////////////////////
    // Mouse code

    onBoardMouseDown(e) {
        e.preventDefault();
        this.onGenericDragStart(e.clientX, e.clientY);
    }

    onBoardMouseMove(e) {
        e.preventDefault();
        this.trySyncDragged(e.clientX, e.clientY);
    }

    onBoardMouseUp(e) {
        e.preventDefault();
        this.onGenericDragEnd(e.clientX, e.clientY);
    }

    //////////////////////////////////////////////////
    // Touch code

    onTouchStart(e) {
        if (e.touches.length === 1) {
            this.onGenericDragStart(e.touches[0].clientX, e.touches[0].clientY);
        }
    }

    onTouchMove(e) {
        if (e.touches.length == 1) {
            e.preventDefault(); // Prevent scroll/zoom while drag
            this.trySyncDragged(e.touches[0].clientX, e.touches[0].clientY);
        }
    }

    onTouchEnd(e) {
        if (e.touches.length === 0 && e.changedTouches.length === 1) {
            this.onGenericDragEnd(e.changedTouches[0].clientX, e.changedTouches[0].clientY);
        } else {
            this.onGenericDragEnd(-1, -1);
        }
    }

    updateEvaluation(evalScore) {
        if (evalScore === null || evalScore === undefined) {
            this.evalText.textContent = '?';
            this.updateEvalBarBackground(0);
            return;
        }

        const displayEval = (evalScore / 100).toFixed(1);
        this.evalText.textContent = evalScore >= 0 ? `+${displayEval}` : displayEval;
        
        this.updateEvalBarBackground(evalScore);
    }

    updateEvalBarBackground(evalScore) {
        evalScore = Math.max(-1000, Math.min(1000, evalScore));
        const percentage = ((evalScore + 1000) / 2000) * 100;
        
        if (this.isWhiteCameraPosition) {
            this.evalBar.style.background = `linear-gradient(to top, #fff 0%, #fff ${percentage}%, #000 ${percentage}%, #000 100%)`;
        } else {
            this.evalBar.style.background = `linear-gradient(to top, #000 0%, #000 ${100 - percentage}%, #fff ${100 - percentage}%, #fff 100%)`;
        }
    }

    //////////////////////////////////////////////////
    // Purely UI code

    setSquareFromWasmData(row, col, data) {
        const existing = this.lastBoardState[row * 8 + col];
        const num = this.isWhiteCameraPosition ? data[row * 8 + col] : data[(7 - row) * 8 + (7 - col)];
        if (existing === num) {
            this.colorSquare(row, col, false);
        } else {
            if (num === 0) {
                this.setSquare(row, col, null);
            } else {
                const isWhite = num > 0;
                const letter = this.numToLetter[Math.abs(num) - 1];
                if (letter !== undefined) this.setSquare(row, col, letter, isWhite);
            }
            this.lastBoardState[row * 8 + col] = num;
            if (existing !== undefined) this.colorSquare(row, col, true); // Don't color on first sync from undefined -> number
        }
        return num;
    }

    colorSquare(row, col, isColored) {
        const imageRow = this.squareImages[row];
        if (imageRow === undefined) return;
        const image = imageRow[col];
        if (image === undefined) return;

        if (isColored) {
            image.parentElement.style.backgroundColor = '#a33c2c';
        } else {
            image.parentElement.style.backgroundColor = image.parentElement.dataset.backgroundColor;        
        }
    }

    setSquare(row, col, code, isWhite) {
        const src = typeof code === 'string' ? imageUrls[code.toLowerCase() + (isWhite ? 'w' : 'b')] : null;
        return this._setSquare(row, col, src);
    }

    _setSquare(row, col, src) {
        const imageRow = this.squareImages[row];
        if (imageRow === undefined) return;
        const image = imageRow[col];
        if (image === undefined) return;

        if (src) {
            image.src = src;
            image.style.visibility = 'visible';
        } else {
            image.src = '';
            image.style.visibility = 'hidden';
        }
    }

    refreshBoardFromWasmData(data) {
        for (let y = 0; y < 8; ++y) {
            for (let x = 0; x < 8; ++x) {
                this.setSquareFromWasmData(x, y, data);
            }
        }
    }

    redrawMoveList() {
        const moveListContent = /** @type {HTMLTextAreaElement} */ (document.getElementById('move-list-content'));
        moveListContent.value = '';
        for (let i = 0; i < this.moveHistory.length; i += 2) {
            
            const moveNumber = Math.floor(i / 2) + 1;
            let text = `${moveNumber}. `;
            
            if (this.moveHistory[i]) {
                text += this.moveHistory[i];
            }
            
            if (this.moveHistory[i + 1]) {
                text += ` ${this.moveHistory[i + 1]}`;
            }
            moveListContent.value += text;
            moveListContent.value += '\n';
        }
        
        moveListContent.scrollTop = moveListContent.scrollHeight;
    }

    addLastMoveToHistory(lastMoveStr) {
        if (lastMoveStr) {
            this.moveHistory.push(lastMoveStr);
            this.redrawMoveList();
            return true;
        } else {
            // This will happen if you checkmate the AI
            console.log('No more last move');
            return false;
        }
    }

    //////////////////////////////////////////////////
    // Coordinate utils

    getBoardCoordsFromClientCoords(clientX, clientY) {
        const r = this.board.getBoundingClientRect();
        return {x: clientX - r.left, y: clientY - r.top};
    }

    getSquareCoordsFromClientCoords(clientX, clientY) {
        const r = this.getBoardCoordsFromClientCoords(clientX, clientY);
        r.x = (r.x / this.SQUARE_LENGTH) >>> 0;
        r.y = (r.y / this.SQUARE_LENGTH) >>> 0;
        return r;
    }
}

new Application();
