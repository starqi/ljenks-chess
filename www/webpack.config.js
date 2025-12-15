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
            //forceMode: "development",
        }),
        /*
        TODO What is this?
        new webpack.ProvidePlugin({
            TextDecoder: ['text-encoding', 'TextDecoder'],
            TextEncoder: ['text-encoding', 'TextEncoder']
        })
        */
    ],
    //mode: 'development',
    mode: 'production',
    // Enabling this slowed down NPS by 4x
    //devtool: 'source-map',
    experiments: {
        asyncWebAssembly: true // TODO Review
    },
    module: {
        rules: [
            { test: /\.png$/i, type: 'asset/resource' }
        ]
    }
};
