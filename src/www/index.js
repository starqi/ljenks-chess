
class Board {

    LEN = 85;

    constructor() {
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

    setSquare(row, col, code, isWhite) {
        const src = typeof code === 'string' ? 'assets/' + code.toLowerCase() + (isWhite ? 'w' : 'b') + '.png' : null;
        return this._setSquare(row, col, src);
    }

    _setSquare(row, col, src) {
        const image = this.squareImages[row][col];
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
    board.setSquare((Math.random() * 8) >>> 0, (Math.random() * 8) >>> 0, 'B', Math.random() > 0.5);
    board.setSquare((Math.random() * 8) >>> 0, (Math.random() * 8) >>> 0, null);
    board.setSquare((Math.random() * 8) >>> 0, (Math.random() * 8) >>> 0, 'N', Math.random() > 0.5);
    board.setSquare((Math.random() * 8) >>> 0, (Math.random() * 8) >>> 0, null);
    board.setSquare((Math.random() * 8) >>> 0, (Math.random() * 8) >>> 0, 'P', Math.random() > 0.5);
    board.setSquare((Math.random() * 8) >>> 0, (Math.random() * 8) >>> 0, null);
    board.setSquare((Math.random() * 8) >>> 0, (Math.random() * 8) >>> 0, 'q', Math.random() > 0.5);
    board.setSquare((Math.random() * 8) >>> 0, (Math.random() * 8) >>> 0, null);
}, 1000);
