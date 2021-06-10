
import * as wasm from './node_modules/chess_bs';

class Board {
    constructor() {

        this.draggedImage = null;

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

        for (let i = 0; i < 8; ++i) {
            const rowElement = document.createElement('div');
            const imageRow = [];
            const delta = i % 2 === 0 ? 0 : 1;

            for (let i = 0; i < 8; ++i) {
                const square = document.createElement('span');
                square.style.width = this.LEN;
                square.style.height = this.LEN;
                square.style.display = 'inline-block';
                square.style.backgroundColor = (i + delta) % 2 === 0 ? '#aaaaaa' : '#333333';

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
    }

    onBoardMouseDown(e) {
        e.preventDefault();

        const r = this.board.getBoundingClientRect();
        const boardX = e.clientX - r.left;
        const boardY = e.clientY - r.top;

        const squareX = (boardX / this.LEN) >>> 0;
        const squareY = (boardY / this.LEN) >>> 0;

        const row = this.squareImages[squareY];
        if (row === undefined) return;
        const image = row[squareX];
        if (image === undefined || image.style.visibility === 'hidden') return;

        image.style.visibility = 'hidden';
        this.draggedImage = image;
        this.dragged.src = image.src;
        this.dragged.style.visibility = 'visible';
        this.trySyncDragged(e);
    }

    trySyncDragged(e) {
        if (this.draggedImage !== null) {
            const r = this.board.getBoundingClientRect();
            const boardX = e.clientX - r.left;
            const boardY = e.clientY - r.top;

            this.dragged.style.left = boardX - this.LEN / 2.0;
            this.dragged.style.top = boardY - this.LEN / 2.0;
        }
    }

    onBoardMouseMove(e) {
        e.preventDefault();
        this.trySyncDragged(e);
    }

    onBoardMouseUp(e) {
        e.preventDefault();

        this.draggedImage.style.visibility = 'visible';
        this.draggedImage = null;
        this.dragged.style.visibility = 'hidden';
    }

    setSquareFromWasm(row, col) {
        const num = board.main.get_piece(col, row);
        if (num === 0) {
            this.setSquare(row, col, null);
        } else {
            const isWhite = num > 0;
            const letter = this.numToLetter[Math.abs(num) - 1];
            if (letter !== undefined) this.setSquare(row, col, letter, isWhite);
        }
    }

    setSquare(row, col, code, isWhite) {
        const src = typeof code === 'string' ? 'assets/' + code.toLowerCase() + (isWhite ? 'w' : 'b') + '.png' : null;
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
}

const board = new Board();
board.updateFromWasm();
setInterval(() => {
    board.main.make_move();
    board.updateFromWasm();
}, 1000);
