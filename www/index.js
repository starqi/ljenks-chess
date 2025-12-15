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
        // If engine is thinking, lock all moves, cannot simply buffer multiple human actions because engine needs to respond  
        // between each premove and then redraw.
        this.boardActionLock = true;

        this.draggedImage = null;
        this.draggedSqX = 0;
        this.draggedSqY = 0;

        this.moveHistory = [];
        this.moveNumber = 1;

        this.SQUARE_LENGTH = (0.9 * Math.min(window.innerWidth, window.innerHeight - document.getElementById('title').getBoundingClientRect().height) / 8) >>> 0;

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

        document.getElementById('self-play-btn').addEventListener('click', () => {
            this.onSelfPlayButtonClick();
        });
        document.getElementById('play-btn').addEventListener('click', () => {
            this.onPlayButtonClick();
        });

        this.dragged = document.getElementById('dragged');
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
                square.style.width = this.SQUARE_LENGTH;
                square.style.height = this.SQUARE_LENGTH;
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
        this.onPlayButtonClick();
    }

    //////////////////////////////////////////////////
    // Web worker interface

    reset(onReady) {
        if (this.worker) {
            this.worker.terminate();
            this.worker = null;
        }
        this.boardActionLock = true;

        console.log('Creating new worker');
        this.worker = new Worker(new URL('worker.js', import.meta.url));
        this.worker.onerror = (e) => {
            console.error('Worker error: ', e);
        };
        let cbCalled = false;
        this.worker.onmessage = (e) => {
            console.log('Worker response', e.data);

            if (e.data.type === 'ready') {
                if (!cbCalled) {
                    this.isWhiteCameraPosition = Math.random() > 0.5;
                    // Twice to get rid of board "diffs" between old and new boards
                    this.refreshBoardFromWasmData(e.data.board);
                    this.refreshBoardFromWasmData(e.data.board);

                    this.moveHistory = [];
                    this.moveNumber = 1;
                    this.redrawMoveList();

                    onReady();
                    cbCalled = true;
                    this.boardActionLock = false;
                }
            } else if (e.data.type === 'ai_move_done') {
                this.addLastMoveToHistory(e.data.lastMoveStr);
                this.refreshBoardFromWasmData(e.data.board);
                this.boardActionLock = false;

                if (e.data.isAutoPlay) this.scheduleSelfPlayChain();
            } else if (e.data.type === 'human_move_done') {
                this.addLastMoveToHistory(e.data.lastMoveStr);
                this.refreshBoardFromWasmData(e.data.board);
                this.boardActionLock = false;

                if (e.data.isAutoPlay) this.makeAiMoveAsync(Application.TEMP_DEPTH, false);
            } else {
                // For errors and other unknown cases, don't perma lock the board
                this.boardActionLock = false;
            }
        };
    }

    makeAiMoveAsync(depth, isAutoPlay) {
        this.boardActionLock = true;
        this.worker.postMessage({type: 'make_ai_move', depth, isAutoPlay});
    }

    makeHumanMoveAsync(fromX, fromY, toX, toY, isAutoPlay) {
        this.boardActionLock = true;
        this.worker.postMessage({type: 'make_human_move', fromX, fromY, toX, toY, isAutoPlay});
    }

    onPlayButtonClick() {
        this.reset(() => {
            if (!this.isWhiteCameraPosition) {
                this.makeAiMoveAsync(Application.TEMP_DEPTH, false);
            }
        });
    }

    onSelfPlayButtonClick() {
        this.reset(() => {
            this.scheduleSelfPlayChain();
        });
    }

    scheduleSelfPlayChain() {
        this.makeAiMoveAsync(Application.TEMP_DEPTH, true);
    }

    //////////////////////////////////////////////////
    // Generic piece move (touch, mouse) code

    onGenericDragStart(clientX, clientY) {

        if (this.draggedImage !== null) {
            // Contract = draggedImage is synced to null if mouse up
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
        if (this.draggedImage === null) return;

        this.draggedImage.style.visibility = 'visible';
        this.draggedImage = null;
        this.dragged.style.visibility = 'hidden';

        const sqCoords = this.getSquareCoordsFromClientCoords(clientX, clientY);
        if (this.boardActionLock) return; // TODO Premoves
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
        const moveListContent = document.getElementById('move-list-content');
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
