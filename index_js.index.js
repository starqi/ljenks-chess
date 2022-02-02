"use strict";
/*
 * ATTENTION: The "eval" devtool has been used (maybe by default in mode: "development").
 * This devtool is neither made for production nor for readable output files.
 * It uses "eval()" calls to create a separate source file in the browser devtools.
 * If you are trying to read the output file, select a different devtool (https://webpack.js.org/configuration/devtool/)
 * or disable the default devtool with "devtool: false".
 * If you are looking for production-ready output files, see mode: "production" (https://webpack.js.org/configuration/mode/).
 */
(self["webpackChunkljenks_chess"] = self["webpackChunkljenks_chess"] || []).push([["index_js"],{

/***/ "../pkg/index_bg.js":
/*!**************************!*\
  !*** ../pkg/index_bg.js ***!
  \**************************/
/***/ ((module, __webpack_exports__, __webpack_require__) => {

eval("__webpack_require__.r(__webpack_exports__);\n/* harmony export */ __webpack_require__.d(__webpack_exports__, {\n/* harmony export */   \"Main\": () => (/* binding */ Main),\n/* harmony export */   \"__wbg_log_7a537c26c1d822a4\": () => (/* binding */ __wbg_log_7a537c26c1d822a4),\n/* harmony export */   \"__wbg_random_bceee7767cab56a7\": () => (/* binding */ __wbg_random_bceee7767cab56a7),\n/* harmony export */   \"__wbg_now_46bdbd53520aa0f6\": () => (/* binding */ __wbg_now_46bdbd53520aa0f6),\n/* harmony export */   \"__wbg_error_09919627ac0992f5\": () => (/* binding */ __wbg_error_09919627ac0992f5),\n/* harmony export */   \"__wbg_new_693216e109162396\": () => (/* binding */ __wbg_new_693216e109162396),\n/* harmony export */   \"__wbg_stack_0ddaca5d1abfb52f\": () => (/* binding */ __wbg_stack_0ddaca5d1abfb52f),\n/* harmony export */   \"__wbindgen_object_drop_ref\": () => (/* binding */ __wbindgen_object_drop_ref),\n/* harmony export */   \"__wbindgen_throw\": () => (/* binding */ __wbindgen_throw)\n/* harmony export */ });\n/* harmony import */ var _index_bg_wasm__WEBPACK_IMPORTED_MODULE_0__ = __webpack_require__(/*! ./index_bg.wasm */ \"../pkg/index_bg.wasm\");\n/* module decorator */ module = __webpack_require__.hmd(module);\n\n\nconst heap = new Array(32).fill(undefined);\n\nheap.push(undefined, null, true, false);\n\nfunction getObject(idx) { return heap[idx]; }\n\nlet heap_next = heap.length;\n\nfunction dropObject(idx) {\n    if (idx < 36) return;\n    heap[idx] = heap_next;\n    heap_next = idx;\n}\n\nfunction takeObject(idx) {\n    const ret = getObject(idx);\n    dropObject(idx);\n    return ret;\n}\n\nconst lTextDecoder = typeof TextDecoder === 'undefined' ? (0, module.require)('util').TextDecoder : TextDecoder;\n\nlet cachedTextDecoder = new lTextDecoder('utf-8', { ignoreBOM: true, fatal: true });\n\ncachedTextDecoder.decode();\n\nlet cachegetUint8Memory0 = null;\nfunction getUint8Memory0() {\n    if (cachegetUint8Memory0 === null || cachegetUint8Memory0.buffer !== _index_bg_wasm__WEBPACK_IMPORTED_MODULE_0__.memory.buffer) {\n        cachegetUint8Memory0 = new Uint8Array(_index_bg_wasm__WEBPACK_IMPORTED_MODULE_0__.memory.buffer);\n    }\n    return cachegetUint8Memory0;\n}\n\nfunction getStringFromWasm0(ptr, len) {\n    return cachedTextDecoder.decode(getUint8Memory0().subarray(ptr, ptr + len));\n}\n\nfunction logError(f, args) {\n    try {\n        return f.apply(this, args);\n    } catch (e) {\n        let error = (function () {\n            try {\n                return e instanceof Error ? `${e.message}\\n\\nStack:\\n${e.stack}` : e.toString();\n            } catch(_) {\n                return \"<failed to stringify thrown value>\";\n            }\n        }());\n        console.error(\"wasm-bindgen: imported JS function that was not marked as `catch` threw an error:\", error);\n        throw e;\n    }\n}\n\nfunction notDefined(what) { return () => { throw new Error(`${what} is not defined`); }; }\n\nfunction _assertNum(n) {\n    if (typeof(n) !== 'number') throw new Error('expected a number argument');\n}\n\nfunction addHeapObject(obj) {\n    if (heap_next === heap.length) heap.push(heap.length + 1);\n    const idx = heap_next;\n    heap_next = heap[idx];\n\n    if (typeof(heap_next) !== 'number') throw new Error('corrupt heap');\n\n    heap[idx] = obj;\n    return idx;\n}\n\nlet WASM_VECTOR_LEN = 0;\n\nconst lTextEncoder = typeof TextEncoder === 'undefined' ? (0, module.require)('util').TextEncoder : TextEncoder;\n\nlet cachedTextEncoder = new lTextEncoder('utf-8');\n\nconst encodeString = (typeof cachedTextEncoder.encodeInto === 'function'\n    ? function (arg, view) {\n    return cachedTextEncoder.encodeInto(arg, view);\n}\n    : function (arg, view) {\n    const buf = cachedTextEncoder.encode(arg);\n    view.set(buf);\n    return {\n        read: arg.length,\n        written: buf.length\n    };\n});\n\nfunction passStringToWasm0(arg, malloc, realloc) {\n\n    if (typeof(arg) !== 'string') throw new Error('expected a string argument');\n\n    if (realloc === undefined) {\n        const buf = cachedTextEncoder.encode(arg);\n        const ptr = malloc(buf.length);\n        getUint8Memory0().subarray(ptr, ptr + buf.length).set(buf);\n        WASM_VECTOR_LEN = buf.length;\n        return ptr;\n    }\n\n    let len = arg.length;\n    let ptr = malloc(len);\n\n    const mem = getUint8Memory0();\n\n    let offset = 0;\n\n    for (; offset < len; offset++) {\n        const code = arg.charCodeAt(offset);\n        if (code > 0x7F) break;\n        mem[ptr + offset] = code;\n    }\n\n    if (offset !== len) {\n        if (offset !== 0) {\n            arg = arg.slice(offset);\n        }\n        ptr = realloc(ptr, len, len = offset + arg.length * 3);\n        const view = getUint8Memory0().subarray(ptr + offset, ptr + len);\n        const ret = encodeString(arg, view);\n        if (ret.read !== arg.length) throw new Error('failed to pass whole string');\n        offset += ret.written;\n    }\n\n    WASM_VECTOR_LEN = offset;\n    return ptr;\n}\n\nlet cachegetInt32Memory0 = null;\nfunction getInt32Memory0() {\n    if (cachegetInt32Memory0 === null || cachegetInt32Memory0.buffer !== _index_bg_wasm__WEBPACK_IMPORTED_MODULE_0__.memory.buffer) {\n        cachegetInt32Memory0 = new Int32Array(_index_bg_wasm__WEBPACK_IMPORTED_MODULE_0__.memory.buffer);\n    }\n    return cachegetInt32Memory0;\n}\n/**\n*/\nclass Main {\n\n    constructor() {\n        throw new Error('cannot invoke `new` directly');\n    }\n\n    static __wrap(ptr) {\n        const obj = Object.create(Main.prototype);\n        obj.ptr = ptr;\n\n        return obj;\n    }\n\n    __destroy_into_raw() {\n        const ptr = this.ptr;\n        this.ptr = 0;\n\n        return ptr;\n    }\n\n    free() {\n        const ptr = this.__destroy_into_raw();\n        _index_bg_wasm__WEBPACK_IMPORTED_MODULE_0__.__wbg_main_free(ptr);\n    }\n    /**\n    * @returns {Main}\n    */\n    static new() {\n        var ret = _index_bg_wasm__WEBPACK_IMPORTED_MODULE_0__.main_new();\n        return Main.__wrap(ret);\n    }\n    /**\n    */\n    make_ai_move() {\n        if (this.ptr == 0) throw new Error('Attempt to use a moved value');\n        _assertNum(this.ptr);\n        _index_bg_wasm__WEBPACK_IMPORTED_MODULE_0__.main_make_ai_move(this.ptr);\n    }\n    /**\n    */\n    refresh_player_moves() {\n        if (this.ptr == 0) throw new Error('Attempt to use a moved value');\n        _assertNum(this.ptr);\n        _index_bg_wasm__WEBPACK_IMPORTED_MODULE_0__.main_refresh_player_moves(this.ptr);\n    }\n    /**\n    * @param {number} from_x\n    * @param {number} from_y\n    * @param {number} to_x\n    * @param {number} to_y\n    * @returns {boolean}\n    */\n    try_move(from_x, from_y, to_x, to_y) {\n        if (this.ptr == 0) throw new Error('Attempt to use a moved value');\n        _assertNum(this.ptr);\n        _assertNum(from_x);\n        _assertNum(from_y);\n        _assertNum(to_x);\n        _assertNum(to_y);\n        var ret = _index_bg_wasm__WEBPACK_IMPORTED_MODULE_0__.main_try_move(this.ptr, from_x, from_y, to_x, to_y);\n        return ret !== 0;\n    }\n    /**\n    * @param {number} x\n    * @param {number} y\n    * @returns {number}\n    */\n    get_piece(x, y) {\n        if (this.ptr == 0) throw new Error('Attempt to use a moved value');\n        _assertNum(this.ptr);\n        _assertNum(x);\n        _assertNum(y);\n        var ret = _index_bg_wasm__WEBPACK_IMPORTED_MODULE_0__.main_get_piece(this.ptr, x, y);\n        return ret;\n    }\n}\n\nfunction __wbg_log_7a537c26c1d822a4() { return logError(function (arg0, arg1) {\n    console.log(getStringFromWasm0(arg0, arg1));\n}, arguments) };\n\nconst __wbg_random_bceee7767cab56a7 = typeof Math.random == 'function' ? Math.random : notDefined('Math.random');\n\nconst __wbg_now_46bdbd53520aa0f6 = typeof Date.now == 'function' ? Date.now : notDefined('Date.now');\n\nfunction __wbg_error_09919627ac0992f5() { return logError(function (arg0, arg1) {\n    try {\n        console.error(getStringFromWasm0(arg0, arg1));\n    } finally {\n        _index_bg_wasm__WEBPACK_IMPORTED_MODULE_0__.__wbindgen_free(arg0, arg1);\n    }\n}, arguments) };\n\nfunction __wbg_new_693216e109162396() { return logError(function () {\n    var ret = new Error();\n    return addHeapObject(ret);\n}, arguments) };\n\nfunction __wbg_stack_0ddaca5d1abfb52f() { return logError(function (arg0, arg1) {\n    var ret = getObject(arg1).stack;\n    var ptr0 = passStringToWasm0(ret, _index_bg_wasm__WEBPACK_IMPORTED_MODULE_0__.__wbindgen_malloc, _index_bg_wasm__WEBPACK_IMPORTED_MODULE_0__.__wbindgen_realloc);\n    var len0 = WASM_VECTOR_LEN;\n    getInt32Memory0()[arg0 / 4 + 1] = len0;\n    getInt32Memory0()[arg0 / 4 + 0] = ptr0;\n}, arguments) };\n\nfunction __wbindgen_object_drop_ref(arg0) {\n    takeObject(arg0);\n};\n\nfunction __wbindgen_throw(arg0, arg1) {\n    throw new Error(getStringFromWasm0(arg0, arg1));\n};\n\n\n\n//# sourceURL=webpack://ljenks-chess/../pkg/index_bg.js?");

/***/ }),

/***/ "./index.js":
/*!******************!*\
  !*** ./index.js ***!
  \******************/
/***/ ((__unused_webpack_module, __webpack_exports__, __webpack_require__) => {

eval("__webpack_require__.r(__webpack_exports__);\n/* harmony import */ var _node_modules_ljenks_chess__WEBPACK_IMPORTED_MODULE_12__ = __webpack_require__(/*! ./node_modules/ljenks-chess */ \"../pkg/index_bg.js\");\n/* harmony import */ var _assets_bb_png__WEBPACK_IMPORTED_MODULE_0__ = __webpack_require__(/*! ./assets/bb.png */ \"./assets/bb.png\");\n/* harmony import */ var _assets_bw_png__WEBPACK_IMPORTED_MODULE_1__ = __webpack_require__(/*! ./assets/bw.png */ \"./assets/bw.png\");\n/* harmony import */ var _assets_kb_png__WEBPACK_IMPORTED_MODULE_2__ = __webpack_require__(/*! ./assets/kb.png */ \"./assets/kb.png\");\n/* harmony import */ var _assets_kw_png__WEBPACK_IMPORTED_MODULE_3__ = __webpack_require__(/*! ./assets/kw.png */ \"./assets/kw.png\");\n/* harmony import */ var _assets_nb_png__WEBPACK_IMPORTED_MODULE_4__ = __webpack_require__(/*! ./assets/nb.png */ \"./assets/nb.png\");\n/* harmony import */ var _assets_nw_png__WEBPACK_IMPORTED_MODULE_5__ = __webpack_require__(/*! ./assets/nw.png */ \"./assets/nw.png\");\n/* harmony import */ var _assets_pb_png__WEBPACK_IMPORTED_MODULE_6__ = __webpack_require__(/*! ./assets/pb.png */ \"./assets/pb.png\");\n/* harmony import */ var _assets_pw_png__WEBPACK_IMPORTED_MODULE_7__ = __webpack_require__(/*! ./assets/pw.png */ \"./assets/pw.png\");\n/* harmony import */ var _assets_qb_png__WEBPACK_IMPORTED_MODULE_8__ = __webpack_require__(/*! ./assets/qb.png */ \"./assets/qb.png\");\n/* harmony import */ var _assets_qw_png__WEBPACK_IMPORTED_MODULE_9__ = __webpack_require__(/*! ./assets/qw.png */ \"./assets/qw.png\");\n/* harmony import */ var _assets_rb_png__WEBPACK_IMPORTED_MODULE_10__ = __webpack_require__(/*! ./assets/rb.png */ \"./assets/rb.png\");\n/* harmony import */ var _assets_rw_png__WEBPACK_IMPORTED_MODULE_11__ = __webpack_require__(/*! ./assets/rw.png */ \"./assets/rw.png\");\n\n\n\n\n\n\n\n\n\n\n\n\n\n\n\nconst imageUrls = {\n    bb: _assets_bb_png__WEBPACK_IMPORTED_MODULE_0__[\"default\"], bw: _assets_bw_png__WEBPACK_IMPORTED_MODULE_1__[\"default\"], kb: _assets_kb_png__WEBPACK_IMPORTED_MODULE_2__[\"default\"], kw: _assets_kw_png__WEBPACK_IMPORTED_MODULE_3__[\"default\"], nb: _assets_nb_png__WEBPACK_IMPORTED_MODULE_4__[\"default\"], nw: _assets_nw_png__WEBPACK_IMPORTED_MODULE_5__[\"default\"], pb: _assets_pb_png__WEBPACK_IMPORTED_MODULE_6__[\"default\"], pw: _assets_pw_png__WEBPACK_IMPORTED_MODULE_7__[\"default\"], qb: _assets_qb_png__WEBPACK_IMPORTED_MODULE_8__[\"default\"], qw: _assets_qw_png__WEBPACK_IMPORTED_MODULE_9__[\"default\"], rb: _assets_rb_png__WEBPACK_IMPORTED_MODULE_10__[\"default\"], rw: _assets_rw_png__WEBPACK_IMPORTED_MODULE_11__[\"default\"]\n};\n\nclass Application {\n    constructor() {\n\n        this.boardLock = false;\n\n        this.draggedImage = null;\n        this.draggedSqX = 0;\n        this.draggedSqY = 0;\n\n        this.main = _node_modules_ljenks_chess__WEBPACK_IMPORTED_MODULE_12__.Main[\"new\"]();\n        this.LEN = (0.9 * Math.min(window.innerWidth, window.innerHeight - document.getElementById('title').getBoundingClientRect().height) / 8) >>> 0;\n\n        // Pawn = 0, Rook, Knight, Bishop, Queen, King\n        this.numToLetter = [\n            'p', 'r', 'n', 'b', 'q', 'k'\n        ];\n\n        this.board = document.getElementById('board');\n        this.board.addEventListener('mousedown', this.onBoardMouseDown.bind(this));\n        this.board.addEventListener('mousemove', this.onBoardMouseMove.bind(this));\n        this.board.addEventListener('mouseup', this.onBoardMouseUp.bind(this));\n        this.board.addEventListener('touchstart', this.onTouchStart.bind(this));\n        this.board.addEventListener('touchmove', this.onTouchMove.bind(this));\n        this.board.addEventListener('touchend', this.onTouchEnd.bind(this));\n\n        this.dragged = document.getElementById('dragged');\n        this.dragged.width = this.LEN;\n        this.dragged.height = this.LEN;\n\n        this.squareImages = [];\n        this.wasmData = new Array(64);\n\n        for (let i = 0; i < 8; ++i) {\n            const rowElement = document.createElement('div');\n            const imageRow = [];\n            const delta = i % 2 === 0 ? 0 : 1;\n\n            for (let i = 0; i < 8; ++i) {\n                const square = document.createElement('span');\n                square.style.width = this.LEN;\n                square.style.height = this.LEN;\n                square.style.display = 'inline-block';\n                square.style.backgroundColor = (i + delta) % 2 === 0 ? '#eeeeee' : '#915355';\n                square.dataset.backgroundColor = square.style.backgroundColor;\n\n                const image = new Image();\n                image.width = this.LEN;\n                image.height = this.LEN;\n                image.style.visibility = 'hidden';\n                image.src = '';\n\n                square.append(image);\n                rowElement.append(square);\n                imageRow.push(image);\n            }\n\n            this.board.append(rowElement);\n            this.squareImages.push(imageRow);\n        }\n\n        this.isPlayerWhite = Math.random() > 0.5;\n        if (!this.isPlayerWhite) {\n            this.main.make_ai_move();\n        }\n        this.main.refresh_player_moves();\n        this.updateFromWasm();\n    }\n\n    //////////////////////////////////////////////////\n\n    onGenericDragStart(clientX, clientY) {\n        if (this.boardLock) return;\n\n        if (this.draggedImage !== null) {\n            // Contract = draggedImage is synced to null if mouse up\n            // But if any shenanigans with mouse/touch up not being called, then clean up the invisible piece\n            this.draggedImage.style.visibility = 'visible';\n        }\n\n        const sqCoords = this.getSquareCoordsFromClientCoords(clientX, clientY);\n\n        const row = this.squareImages[sqCoords.y];\n        if (row === undefined) return;\n        const image = row[sqCoords.x];\n        if (image === undefined || image.style.visibility === 'hidden') return;\n\n        image.style.visibility = 'hidden';\n        this.draggedImage = image;\n        this.dragged.src = image.src;\n        this.dragged.style.visibility = 'visible';\n        this.draggedSqY = sqCoords.y;\n        this.draggedSqX = sqCoords.x;\n        this.trySyncDragged(clientX, clientY);\n    }\n\n    trySyncDragged(clientX, clientY) {\n        const boardCoords = this.getBoardCoordsFromClientCoords(clientX, clientY);\n        if (this.draggedImage !== null) {\n            this.dragged.style.left = boardCoords.x - this.LEN / 2.0;\n            this.dragged.style.top = boardCoords.y - this.LEN / 2.0;\n        }\n    }\n\n    onGenericDragEnd(clientX, clientY) {\n        if (this.draggedImage === null) return;\n\n        this.draggedImage.style.visibility = 'visible';\n        this.draggedImage = null;\n        this.dragged.style.visibility = 'hidden';\n\n        const sqCoords = this.getSquareCoordsFromClientCoords(clientX, clientY);\n        if (this.isPlayerWhite) {\n            if (!this.main.try_move(\n                this.draggedSqX,\n                this.draggedSqY,\n                sqCoords.x,\n                sqCoords.y\n            )) return;\n        } else {\n            if (!this.main.try_move(\n                7 - this.draggedSqX,\n                7 - this.draggedSqY,\n                7 - sqCoords.x,\n                7 - sqCoords.y\n            )) return;\n        }\n\n        this.updateFromWasm();\n\n        this.boardLock = true;\n        console.log('Locked board');\n        setTimeout(() => {\n            this.main.make_ai_move();\n            this.updateFromWasm();\n            this.main.refresh_player_moves();\n            this.boardLock = false;\n            console.log('Unlocked board');\n        }, 250);\n    }\n\n    //////////////////////////////////////////////////\n\n    onBoardMouseDown(e) {\n        e.preventDefault();\n        this.onGenericDragStart(e.clientX, e.clientY);\n    }\n\n    onBoardMouseMove(e) {\n        e.preventDefault();\n        this.trySyncDragged(e.clientX, e.clientY);\n    }\n\n    onBoardMouseUp(e) {\n        e.preventDefault();\n        this.onGenericDragEnd(e.clientX, e.clientY);\n    }\n\n    //////////////////////////////////////////////////\n\n    onTouchStart(e) {\n        if (e.touches.length === 1) {\n            this.onGenericDragStart(e.touches[0].clientX, e.touches[0].clientY);\n        }\n    }\n\n    onTouchMove(e) {\n        if (e.touches.length == 1) {\n            e.preventDefault(); // Prevent scroll/zoom while drag\n            this.trySyncDragged(e.touches[0].clientX, e.touches[0].clientY);\n        }\n    }\n\n    onTouchEnd(e) {\n        if (e.touches.length === 0 && e.changedTouches.length === 1) {\n            this.onGenericDragEnd(e.changedTouches[0].clientX, e.changedTouches[0].clientY);\n        } else {\n            this.onGenericDragEnd(-1, -1);\n        }\n    }\n\n    //////////////////////////////////////////////////\n\n    setSquareFromWasm(row, col) {\n        const existing = this.wasmData[row * 8 + col];\n        const num = this.isPlayerWhite ? this.main.get_piece(col, row) : this.main.get_piece(7 - col, 7 - row);\n        if (existing === num) {\n            this.colorSquare(row, col, false);\n        } else {\n            if (num === 0) {\n                this.setSquare(row, col, null);\n            } else {\n                const isWhite = num > 0;\n                const letter = this.numToLetter[Math.abs(num) - 1];\n                if (letter !== undefined) this.setSquare(row, col, letter, isWhite);\n            }\n            this.wasmData[row * 8 + col] = num;\n            if (existing !== undefined) this.colorSquare(row, col, true); // Don't color on first sync from undefined -> number\n        }\n        return num;\n    }\n\n    colorSquare(row, col, isColored) {\n        const imageRow = this.squareImages[row];\n        if (imageRow === undefined) return;\n        const image = imageRow[col];\n        if (image === undefined) return;\n\n        if (isColored) {\n            image.parentElement.style.backgroundColor = '#a33c2c';\n        } else {\n            image.parentElement.style.backgroundColor = image.parentElement.dataset.backgroundColor;        \n        }\n    }\n\n    setSquare(row, col, code, isWhite) {\n        const src = typeof code === 'string' ? imageUrls[code.toLowerCase() + (isWhite ? 'w' : 'b')] : null;\n        return this._setSquare(row, col, src);\n    }\n\n    _setSquare(row, col, src) {\n        const imageRow = this.squareImages[row];\n        if (imageRow === undefined) return;\n        const image = imageRow[col];\n        if (image === undefined) return;\n\n        if (src) {\n            image.src = src;\n            image.style.visibility = 'visible';\n        } else {\n            image.src = '';\n            image.style.visibility = 'hidden';\n        }\n    }\n\n    updateFromWasm() {\n        for (let i = 0; i < 8; ++i) {\n            for (let j = 0; j < 8; ++j) {\n                this.setSquareFromWasm(j, i);\n            }\n        }\n    }\n\n    //////////////////////////////////////////////////\n\n    getBoardCoordsFromClientCoords(clientX, clientY) {\n        const r = this.board.getBoundingClientRect();\n        return {x: clientX - r.left, y: clientY - r.top};\n    }\n\n    getSquareCoordsFromClientCoords(clientX, clientY) {\n        const r = this.getBoardCoordsFromClientCoords(clientX, clientY);\n        r.x = (r.x / this.LEN) >>> 0;\n        r.y = (r.y / this.LEN) >>> 0;\n        return r;\n    }\n}\n\nnew Application();\n\n\n//# sourceURL=webpack://ljenks-chess/./index.js?");

/***/ }),

/***/ "./assets/bb.png":
/*!***********************!*\
  !*** ./assets/bb.png ***!
  \***********************/
/***/ ((__unused_webpack_module, __webpack_exports__, __webpack_require__) => {

eval("__webpack_require__.r(__webpack_exports__);\n/* harmony export */ __webpack_require__.d(__webpack_exports__, {\n/* harmony export */   \"default\": () => (__WEBPACK_DEFAULT_EXPORT__)\n/* harmony export */ });\n/* harmony default export */ const __WEBPACK_DEFAULT_EXPORT__ = (__webpack_require__.p + \"0180c5827bceece87dc3abdae65482ef.png\");\n\n//# sourceURL=webpack://ljenks-chess/./assets/bb.png?");

/***/ }),

/***/ "./assets/bw.png":
/*!***********************!*\
  !*** ./assets/bw.png ***!
  \***********************/
/***/ ((__unused_webpack_module, __webpack_exports__, __webpack_require__) => {

eval("__webpack_require__.r(__webpack_exports__);\n/* harmony export */ __webpack_require__.d(__webpack_exports__, {\n/* harmony export */   \"default\": () => (__WEBPACK_DEFAULT_EXPORT__)\n/* harmony export */ });\n/* harmony default export */ const __WEBPACK_DEFAULT_EXPORT__ = (__webpack_require__.p + \"11f4961ce6da7780b8b871f4b2088514.png\");\n\n//# sourceURL=webpack://ljenks-chess/./assets/bw.png?");

/***/ }),

/***/ "./assets/kb.png":
/*!***********************!*\
  !*** ./assets/kb.png ***!
  \***********************/
/***/ ((__unused_webpack_module, __webpack_exports__, __webpack_require__) => {

eval("__webpack_require__.r(__webpack_exports__);\n/* harmony export */ __webpack_require__.d(__webpack_exports__, {\n/* harmony export */   \"default\": () => (__WEBPACK_DEFAULT_EXPORT__)\n/* harmony export */ });\n/* harmony default export */ const __WEBPACK_DEFAULT_EXPORT__ = (__webpack_require__.p + \"e48feabdd563479eefc8591390c18d1c.png\");\n\n//# sourceURL=webpack://ljenks-chess/./assets/kb.png?");

/***/ }),

/***/ "./assets/kw.png":
/*!***********************!*\
  !*** ./assets/kw.png ***!
  \***********************/
/***/ ((__unused_webpack_module, __webpack_exports__, __webpack_require__) => {

eval("__webpack_require__.r(__webpack_exports__);\n/* harmony export */ __webpack_require__.d(__webpack_exports__, {\n/* harmony export */   \"default\": () => (__WEBPACK_DEFAULT_EXPORT__)\n/* harmony export */ });\n/* harmony default export */ const __WEBPACK_DEFAULT_EXPORT__ = (__webpack_require__.p + \"ae8822a6c7a3fe3d4bc456aac92e680b.png\");\n\n//# sourceURL=webpack://ljenks-chess/./assets/kw.png?");

/***/ }),

/***/ "./assets/nb.png":
/*!***********************!*\
  !*** ./assets/nb.png ***!
  \***********************/
/***/ ((__unused_webpack_module, __webpack_exports__, __webpack_require__) => {

eval("__webpack_require__.r(__webpack_exports__);\n/* harmony export */ __webpack_require__.d(__webpack_exports__, {\n/* harmony export */   \"default\": () => (__WEBPACK_DEFAULT_EXPORT__)\n/* harmony export */ });\n/* harmony default export */ const __WEBPACK_DEFAULT_EXPORT__ = (__webpack_require__.p + \"69534b953181e0c4d0335d4bee6364e7.png\");\n\n//# sourceURL=webpack://ljenks-chess/./assets/nb.png?");

/***/ }),

/***/ "./assets/nw.png":
/*!***********************!*\
  !*** ./assets/nw.png ***!
  \***********************/
/***/ ((__unused_webpack_module, __webpack_exports__, __webpack_require__) => {

eval("__webpack_require__.r(__webpack_exports__);\n/* harmony export */ __webpack_require__.d(__webpack_exports__, {\n/* harmony export */   \"default\": () => (__WEBPACK_DEFAULT_EXPORT__)\n/* harmony export */ });\n/* harmony default export */ const __WEBPACK_DEFAULT_EXPORT__ = (__webpack_require__.p + \"48a5392547eb9403d668d19c1dee23ea.png\");\n\n//# sourceURL=webpack://ljenks-chess/./assets/nw.png?");

/***/ }),

/***/ "./assets/pb.png":
/*!***********************!*\
  !*** ./assets/pb.png ***!
  \***********************/
/***/ ((__unused_webpack_module, __webpack_exports__, __webpack_require__) => {

eval("__webpack_require__.r(__webpack_exports__);\n/* harmony export */ __webpack_require__.d(__webpack_exports__, {\n/* harmony export */   \"default\": () => (__WEBPACK_DEFAULT_EXPORT__)\n/* harmony export */ });\n/* harmony default export */ const __WEBPACK_DEFAULT_EXPORT__ = (__webpack_require__.p + \"de55e4cb5f4b5acf62ee872a3a5b0cca.png\");\n\n//# sourceURL=webpack://ljenks-chess/./assets/pb.png?");

/***/ }),

/***/ "./assets/pw.png":
/*!***********************!*\
  !*** ./assets/pw.png ***!
  \***********************/
/***/ ((__unused_webpack_module, __webpack_exports__, __webpack_require__) => {

eval("__webpack_require__.r(__webpack_exports__);\n/* harmony export */ __webpack_require__.d(__webpack_exports__, {\n/* harmony export */   \"default\": () => (__WEBPACK_DEFAULT_EXPORT__)\n/* harmony export */ });\n/* harmony default export */ const __WEBPACK_DEFAULT_EXPORT__ = (__webpack_require__.p + \"4c3cc6fc5c61c9ac11a2c3476612ee82.png\");\n\n//# sourceURL=webpack://ljenks-chess/./assets/pw.png?");

/***/ }),

/***/ "./assets/qb.png":
/*!***********************!*\
  !*** ./assets/qb.png ***!
  \***********************/
/***/ ((__unused_webpack_module, __webpack_exports__, __webpack_require__) => {

eval("__webpack_require__.r(__webpack_exports__);\n/* harmony export */ __webpack_require__.d(__webpack_exports__, {\n/* harmony export */   \"default\": () => (__WEBPACK_DEFAULT_EXPORT__)\n/* harmony export */ });\n/* harmony default export */ const __WEBPACK_DEFAULT_EXPORT__ = (__webpack_require__.p + \"9cab050258aa5fc071d990ece6e66816.png\");\n\n//# sourceURL=webpack://ljenks-chess/./assets/qb.png?");

/***/ }),

/***/ "./assets/qw.png":
/*!***********************!*\
  !*** ./assets/qw.png ***!
  \***********************/
/***/ ((__unused_webpack_module, __webpack_exports__, __webpack_require__) => {

eval("__webpack_require__.r(__webpack_exports__);\n/* harmony export */ __webpack_require__.d(__webpack_exports__, {\n/* harmony export */   \"default\": () => (__WEBPACK_DEFAULT_EXPORT__)\n/* harmony export */ });\n/* harmony default export */ const __WEBPACK_DEFAULT_EXPORT__ = (__webpack_require__.p + \"7eeecfe963b7301bfc8797e2532bd71b.png\");\n\n//# sourceURL=webpack://ljenks-chess/./assets/qw.png?");

/***/ }),

/***/ "./assets/rb.png":
/*!***********************!*\
  !*** ./assets/rb.png ***!
  \***********************/
/***/ ((__unused_webpack_module, __webpack_exports__, __webpack_require__) => {

eval("__webpack_require__.r(__webpack_exports__);\n/* harmony export */ __webpack_require__.d(__webpack_exports__, {\n/* harmony export */   \"default\": () => (__WEBPACK_DEFAULT_EXPORT__)\n/* harmony export */ });\n/* harmony default export */ const __WEBPACK_DEFAULT_EXPORT__ = (__webpack_require__.p + \"55c06d7782205f329641b3114d5db7e6.png\");\n\n//# sourceURL=webpack://ljenks-chess/./assets/rb.png?");

/***/ }),

/***/ "./assets/rw.png":
/*!***********************!*\
  !*** ./assets/rw.png ***!
  \***********************/
/***/ ((__unused_webpack_module, __webpack_exports__, __webpack_require__) => {

eval("__webpack_require__.r(__webpack_exports__);\n/* harmony export */ __webpack_require__.d(__webpack_exports__, {\n/* harmony export */   \"default\": () => (__WEBPACK_DEFAULT_EXPORT__)\n/* harmony export */ });\n/* harmony default export */ const __WEBPACK_DEFAULT_EXPORT__ = (__webpack_require__.p + \"271e9073e2fb4ece87ef62715f2bdf9e.png\");\n\n//# sourceURL=webpack://ljenks-chess/./assets/rw.png?");

/***/ }),

/***/ "../pkg/index_bg.wasm":
/*!****************************!*\
  !*** ../pkg/index_bg.wasm ***!
  \****************************/
/***/ ((module, exports, __webpack_require__) => {

eval("\"use strict\";\n// Instantiate WebAssembly module\nvar wasmExports = __webpack_require__.w[module.id];\n__webpack_require__.r(exports);\n// export exports from WebAssembly module\nfor(var name in wasmExports) if(name) exports[name] = wasmExports[name];\n// exec imports from WebAssembly module (for esm order)\n/* harmony import */ var m0 = __webpack_require__(/*! ./index_bg.js */ \"../pkg/index_bg.js\");\n\n\n// exec wasm module\nwasmExports[\"\"]()\n\n//# sourceURL=webpack://ljenks-chess/../pkg/index_bg.wasm?");

/***/ })

}]);