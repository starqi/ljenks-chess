import * as wasm from './node_modules/ljenks-chess';
import { initWorker } from './worker_shared.js';

initWorker(new wasm.Main());
