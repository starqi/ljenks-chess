const path = require('path');
const CopyWebpackPlugin = require("copy-webpack-plugin");
const webpack = require('webpack');
const WasmPackPlugin = require("@wasm-tool/wasm-pack-plugin");

module.exports = {
    entry: './loader.js',
    output: {
        path: path.resolve(__dirname, 'dist'),
        filename: 'index.js',
    },
    // Enabling ANY of these slowed down NPS by 4x, why? TODO (Minor)
    //optimization: { // https://webpack.js.org/configuration/optimization/#optimizationminimize
    //    minimize: true
    //},
    //devtool: 'source-map',
    //mode: 'development',
    mode: 'production',
    plugins: [
        new CopyWebpackPlugin({
            patterns: [
                {from: path.resolve(__dirname, 'index.html')},
                {from: path.resolve(__dirname, 'worker.js')},
            ],
        }),
        new WasmPackPlugin({
            crateDirectory: path.resolve(__dirname, ".."),
            forceMode: "production",
            extraArgs: "-- --no-default-features --features wasm",
        }),
    ],
    experiments: {
        asyncWebAssembly: true // AI recommended this is mature enough to be "true"
    },
    module: {
        rules: [
            { test: /\.png$/i, type: 'asset/resource' }
        ]
    }
};
