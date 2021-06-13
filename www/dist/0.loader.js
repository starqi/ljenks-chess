(window["webpackJsonp"] = window["webpackJsonp"] || []).push([[0],{

/***/ "../../pkg/chess_bs.js":
/*!*************************************************************!*\
  !*** C:/Users/starq/OneDrive/Desktop/chess/pkg/chess_bs.js ***!
  \*************************************************************/
/*! exports provided: Main, __wbg_log_94a921ad2284be4b, __wbindgen_throw */
/***/ (function(module, __webpack_exports__, __webpack_require__) {

"use strict";
eval("__webpack_require__.r(__webpack_exports__);\n/* harmony import */ var _chess_bs_bg_wasm__WEBPACK_IMPORTED_MODULE_0__ = __webpack_require__(/*! ./chess_bs_bg.wasm */ \"../../pkg/chess_bs_bg.wasm\");\n/* harmony import */ var _chess_bs_bg_js__WEBPACK_IMPORTED_MODULE_1__ = __webpack_require__(/*! ./chess_bs_bg.js */ \"../../pkg/chess_bs_bg.js\");\n/* harmony reexport (safe) */ __webpack_require__.d(__webpack_exports__, \"Main\", function() { return _chess_bs_bg_js__WEBPACK_IMPORTED_MODULE_1__[\"Main\"]; });\n\n/* harmony reexport (safe) */ __webpack_require__.d(__webpack_exports__, \"__wbg_log_94a921ad2284be4b\", function() { return _chess_bs_bg_js__WEBPACK_IMPORTED_MODULE_1__[\"__wbg_log_94a921ad2284be4b\"]; });\n\n/* harmony reexport (safe) */ __webpack_require__.d(__webpack_exports__, \"__wbindgen_throw\", function() { return _chess_bs_bg_js__WEBPACK_IMPORTED_MODULE_1__[\"__wbindgen_throw\"]; });\n\n\n\n\n//# sourceURL=webpack:///C:/Users/starq/OneDrive/Desktop/chess/pkg/chess_bs.js?");

/***/ }),

/***/ "../../pkg/chess_bs_bg.js":
/*!****************************************************************!*\
  !*** C:/Users/starq/OneDrive/Desktop/chess/pkg/chess_bs_bg.js ***!
  \****************************************************************/
/*! exports provided: Main, __wbg_log_94a921ad2284be4b, __wbindgen_throw */
/***/ (function(module, __webpack_exports__, __webpack_require__) {

"use strict";
eval("__webpack_require__.r(__webpack_exports__);\n/* WEBPACK VAR INJECTION */(function(module) {/* harmony export (binding) */ __webpack_require__.d(__webpack_exports__, \"Main\", function() { return Main; });\n/* harmony export (binding) */ __webpack_require__.d(__webpack_exports__, \"__wbg_log_94a921ad2284be4b\", function() { return __wbg_log_94a921ad2284be4b; });\n/* harmony export (binding) */ __webpack_require__.d(__webpack_exports__, \"__wbindgen_throw\", function() { return __wbindgen_throw; });\n/* harmony import */ var _chess_bs_bg_wasm__WEBPACK_IMPORTED_MODULE_0__ = __webpack_require__(/*! ./chess_bs_bg.wasm */ \"../../pkg/chess_bs_bg.wasm\");\n\n\nconst lTextDecoder = typeof TextDecoder === 'undefined' ? (0, module.require)('util').TextDecoder : TextDecoder;\n\nlet cachedTextDecoder = new lTextDecoder('utf-8', { ignoreBOM: true, fatal: true });\n\ncachedTextDecoder.decode();\n\nlet cachegetUint8Memory0 = null;\nfunction getUint8Memory0() {\n    if (cachegetUint8Memory0 === null || cachegetUint8Memory0.buffer !== _chess_bs_bg_wasm__WEBPACK_IMPORTED_MODULE_0__[\"memory\"].buffer) {\n        cachegetUint8Memory0 = new Uint8Array(_chess_bs_bg_wasm__WEBPACK_IMPORTED_MODULE_0__[\"memory\"].buffer);\n    }\n    return cachegetUint8Memory0;\n}\n\nfunction getStringFromWasm0(ptr, len) {\n    return cachedTextDecoder.decode(getUint8Memory0().subarray(ptr, ptr + len));\n}\n/**\n*/\nclass Main {\n\n    static __wrap(ptr) {\n        const obj = Object.create(Main.prototype);\n        obj.ptr = ptr;\n\n        return obj;\n    }\n\n    __destroy_into_raw() {\n        const ptr = this.ptr;\n        this.ptr = 0;\n\n        return ptr;\n    }\n\n    free() {\n        const ptr = this.__destroy_into_raw();\n        _chess_bs_bg_wasm__WEBPACK_IMPORTED_MODULE_0__[\"__wbg_main_free\"](ptr);\n    }\n    /**\n    * @returns {Main}\n    */\n    static new() {\n        var ret = _chess_bs_bg_wasm__WEBPACK_IMPORTED_MODULE_0__[\"main_new\"]();\n        return Main.__wrap(ret);\n    }\n    /**\n    */\n    make_move() {\n        _chess_bs_bg_wasm__WEBPACK_IMPORTED_MODULE_0__[\"main_make_move\"](this.ptr);\n    }\n    /**\n    * @param {number} x\n    * @param {number} y\n    * @returns {number}\n    */\n    get_piece(x, y) {\n        var ret = _chess_bs_bg_wasm__WEBPACK_IMPORTED_MODULE_0__[\"main_get_piece\"](this.ptr, x, y);\n        return ret;\n    }\n}\n\nfunction __wbg_log_94a921ad2284be4b(arg0, arg1) {\n    console.log(getStringFromWasm0(arg0, arg1));\n};\n\nfunction __wbindgen_throw(arg0, arg1) {\n    throw new Error(getStringFromWasm0(arg0, arg1));\n};\n\n\n/* WEBPACK VAR INJECTION */}.call(this, __webpack_require__(/*! ./../src/www/node_modules/webpack/buildin/harmony-module.js */ \"./node_modules/webpack/buildin/harmony-module.js\")(module)))\n\n//# sourceURL=webpack:///C:/Users/starq/OneDrive/Desktop/chess/pkg/chess_bs_bg.js?");

/***/ }),

/***/ "../../pkg/chess_bs_bg.wasm":
/*!******************************************************************!*\
  !*** C:/Users/starq/OneDrive/Desktop/chess/pkg/chess_bs_bg.wasm ***!
  \******************************************************************/
/*! exports provided: memory, __wbg_main_free, main_new, main_make_move, main_get_piece */
/***/ (function(module, exports, __webpack_require__) {

eval("\"use strict\";\n// Instantiate WebAssembly module\nvar wasmExports = __webpack_require__.w[module.i];\n__webpack_require__.r(exports);\n// export exports from WebAssembly module\nfor(var name in wasmExports) if(name != \"__webpack_init__\") exports[name] = wasmExports[name];\n// exec imports from WebAssembly module (for esm order)\n/* harmony import */ var m0 = __webpack_require__(/*! ./chess_bs_bg.js */ \"../../pkg/chess_bs_bg.js\");\n\n\n// exec wasm module\nwasmExports[\"__webpack_init__\"]()\n\n//# sourceURL=webpack:///C:/Users/starq/OneDrive/Desktop/chess/pkg/chess_bs_bg.wasm?");

/***/ }),

/***/ "./index.js":
/*!******************!*\
  !*** ./index.js ***!
  \******************/
/*! no exports provided */
/***/ (function(module, __webpack_exports__, __webpack_require__) {

"use strict";
eval("__webpack_require__.r(__webpack_exports__);\n/* harmony import */ var _node_modules_chess_bs__WEBPACK_IMPORTED_MODULE_0__ = __webpack_require__(/*! ./node_modules/chess_bs */ \"../../pkg/chess_bs.js\");\n\n\n\nclass Board {\n    constructor() {\n        this.main = _node_modules_chess_bs__WEBPACK_IMPORTED_MODULE_0__[\"Main\"].new();\n        this.LEN = 85;\n\n        // Pawn = 0, Rook, Knight, Bishop, Queen, King\n        this.numToLetter = [\n            'p', 'r', 'n', 'b', 'q', 'k'\n        ];\n\n        this.board = document.getElementById('board');\n        this.squareImages = [];\n\n        for (let i = 0; i < 8; ++i) {\n            const rowElement = document.createElement('div');\n            const imageRow = [];\n            const delta = i % 2 === 0 ? 0 : 1;\n\n            for (let i = 0; i < 8; ++i) {\n                const square = document.createElement('span');\n                square.style.width = this.LEN;\n                square.style.height = this.LEN;\n                square.style.display = 'inline-block';\n                square.style.backgroundColor = (i + delta) % 2 === 0 ? '#aaaaaa' : '#333333';\n\n                const image = new Image();\n                image.width = this.LEN;\n                image.height = this.LEN;\n                image.style.visibility = 'hidden';\n                image.src = '';\n\n                square.append(image);\n                rowElement.append(square);\n                imageRow.push(image);\n            }\n\n            this.board.append(rowElement);\n            this.squareImages.push(imageRow);\n        }\n    }\n\n    setSquareFromWasm(row, col) {\n        const num = board.main.get_piece(col, row);\n        if (num === 0) {\n            this.setSquare(row, col, null);\n        } else {\n            const isWhite = num > 0;\n            const letter = this.numToLetter[Math.abs(num) - 1];\n            if (letter !== undefined) this.setSquare(row, col, letter, isWhite);\n        }\n    }\n\n    setSquare(row, col, code, isWhite) {\n        const src = typeof code === 'string' ? 'assets/' + code.toLowerCase() + (isWhite ? 'w' : 'b') + '.png' : null;\n        return this._setSquare(row, col, src);\n    }\n\n    _setSquare(row, col, src) {\n        const imageRow = this.squareImages[row];\n        if (imageRow === undefined) return;\n        const image = imageRow[col];\n        if (image === undefined) return;\n\n        if (src) {\n            image.src = src;\n            image.style.visibility = 'visible';\n        } else {\n            image.src = '';\n            image.style.visibility = 'hidden';\n        }\n    }\n\n    updateFromWasm() {\n        for (let i = 0; i < 8; ++i) {\n            for (let j = 0; j < 8; ++j) {\n                this.setSquareFromWasm(j, i);\n            }\n        }\n    }\n}\n\nconst board = new Board();\nboard.updateFromWasm();\nsetInterval(() => {\n    board.main.make_move();\n    board.updateFromWasm();\n}, 1000);\n\n\n//# sourceURL=webpack:///./index.js?");

/***/ }),

/***/ "./node_modules/webpack/buildin/harmony-module.js":
/*!*******************************************!*\
  !*** (webpack)/buildin/harmony-module.js ***!
  \*******************************************/
/*! no static exports found */
/***/ (function(module, exports) {

eval("module.exports = function(originalModule) {\n\tif (!originalModule.webpackPolyfill) {\n\t\tvar module = Object.create(originalModule);\n\t\t// module.parent = undefined by default\n\t\tif (!module.children) module.children = [];\n\t\tObject.defineProperty(module, \"loaded\", {\n\t\t\tenumerable: true,\n\t\t\tget: function() {\n\t\t\t\treturn module.l;\n\t\t\t}\n\t\t});\n\t\tObject.defineProperty(module, \"id\", {\n\t\t\tenumerable: true,\n\t\t\tget: function() {\n\t\t\t\treturn module.i;\n\t\t\t}\n\t\t});\n\t\tObject.defineProperty(module, \"exports\", {\n\t\t\tenumerable: true\n\t\t});\n\t\tmodule.webpackPolyfill = 1;\n\t}\n\treturn module;\n};\n\n\n//# sourceURL=webpack:///(webpack)/buildin/harmony-module.js?");

/***/ })

}]);