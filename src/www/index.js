
import * as wasm from './node_modules/chess_bs';

class Board {
    constructor() {
        this.main = wasm.Main.new();
        this.LEN = 85;

        // Pawn = 0, Rook, Knight, Bishop, Queen, King
        this.numToLetter = [
            'p', 'r', 'n', 'b', 'q', 'k'
        ];

        this.board = document.getElementById('board');
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
}

const board = new Board();
setInterval(() => {
    board.main.make_move();

    for (let i = 0; i < 8; ++i) {
        for (let j = 0; j < 8; ++j) {
            board.setSquareFromWasm(j, i);
        }
    }
}, 1000);
