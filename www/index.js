import * as wasm from './node_modules/chess_bs';

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
    constructor() {

        this.boardLock = false;

        this.draggedImage = null;
        this.draggedSqX = 0;
        this.draggedSqY = 0;

        this.main = wasm.Main.new();
        this.LEN = 85;

        // Pawn = 0, Rook, Knight, Bishop, Queen, King
        this.numToLetter = [
            'p', 'r', 'n', 'b', 'q', 'k'
        ];

        this.board = document.getElementById('board');
        this.board.addEventListener('mousedown', this.onBoardMouseDown.bind(this));
        this.board.addEventListener('mousemove', this.onBoardMouseMove.bind(this));
        this.board.addEventListener('mouseup', this.onBoardMouseUp.bind(this));

        this.dragged = document.getElementById('dragged');
        this.dragged.width = this.LEN;
        this.dragged.height = this.LEN;

        this.squareImages = [];
        this.wasmData = new Array(64);

        for (let i = 0; i < 8; ++i) {
            const rowElement = document.createElement('div');
            const imageRow = [];
            const delta = i % 2 === 0 ? 0 : 1;

            for (let i = 0; i < 8; ++i) {
                const square = document.createElement('span');
                square.style.width = this.LEN;
                square.style.height = this.LEN;
                square.style.display = 'inline-block';
                square.style.backgroundColor = (i + delta) % 2 === 0 ? '#eeeeee' : '#539164';
                square.dataset.backgroundColor = square.style.backgroundColor;

                const image = new Image();
                image.width = this.LEN;
                image.height = this.LEN;
                image.style.visibility = 'hidden';
                image.src = '';

                square.append(image);
                rowElement.append(square);
                imageRow.push(image);
            }

            this.board.append(rowElement);
            this.squareImages.push(imageRow);
        }

        this.isPlayerWhite = Math.random() > 0.5;
        if (!this.isPlayerWhite) {
            this.main.make_ai_move();
        }
        this.main.refresh_player_moves();
        this.updateFromWasm();
    }

    onBoardMouseDown(e) {
        e.preventDefault();
        if (this.boardLock) return;

        const sqCoords = this.getSquareCoordsFromMouseEvent(e);

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
        this.trySyncDragged(e);
    }

    trySyncDragged(e) {
        if (this.draggedImage !== null) {
            const clientCoords = this.getBoardCoordsFromMouseEvent(e);

            this.dragged.style.left = clientCoords.x - this.LEN / 2.0;
            this.dragged.style.top = clientCoords.y - this.LEN / 2.0;
        }
    }

    onBoardMouseMove(e) {
        e.preventDefault();
        this.trySyncDragged(e);
    }

    onBoardMouseUp(e) {
        e.preventDefault();

        if (this.draggedImage === null) return;

        this.draggedImage.style.visibility = 'visible';
        this.draggedImage = null;
        this.dragged.style.visibility = 'hidden';

        const sqCoords = this.getSquareCoordsFromMouseEvent(e);
        if (!this.main.try_move(
                this.draggedSqX,
                this.isPlayerWhite ? this.draggedSqY : 7 - this.draggedSqY,
                sqCoords.x,
                this.isPlayerWhite ? sqCoords.y : 7 - sqCoords.y
            )) return;

        this.updateFromWasm();

        this.boardLock = true;
        console.log('Locked board');
        setTimeout(() => {
            this.main.make_ai_move();
            this.updateFromWasm();
            this.main.refresh_player_moves();
            this.boardLock = false;
            console.log('Unlocked board');
        }, 250);
    }

    setSquareFromWasm(row, col) {
        const existing = this.wasmData[row * 8 + col];
        const num = this.main.get_piece(col, this.isPlayerWhite ? row : 7 - row);
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
            this.wasmData[row * 8 + col] = num;
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

    updateFromWasm() {
        for (let i = 0; i < 8; ++i) {
            for (let j = 0; j < 8; ++j) {
                this.setSquareFromWasm(j, i);
            }
        }
    }

    getBoardCoordsFromMouseEvent(e) {
        const r = this.board.getBoundingClientRect();
        return {x: e.clientX - r.left, y: e.clientY - r.top};
    }

    getSquareCoordsFromMouseEvent(e) {
        const r = this.getBoardCoordsFromMouseEvent(e);
        return {
            x: (r.x / this.LEN) >>> 0,
            y: (r.y / this.LEN) >>> 0
        };
    }
}

new Application();
