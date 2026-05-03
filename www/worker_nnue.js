// TODO How does this work as part of build process? i.e. Bindgen questions
import * as wasm from './node_modules/ljenks-chess';
import { initWorker } from './worker_shared.js';

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

async function init() {
    await loadWeights();
    initWorker(new wasm.Main());
}
init();