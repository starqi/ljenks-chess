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
                {
                    from: path.resolve(__dirname, 'index.html'),
                },
            ],
        }),
        new WasmPackPlugin({
            crateDirectory: path.resolve(__dirname, ".."),
            forceMode: "production",
            //forceMode: "development",
        }),
        /*
        new webpack.ProvidePlugin({
            TextDecoder: ['text-encoding', 'TextDecoder'],
            TextEncoder: ['text-encoding', 'TextEncoder']
        })
        */
    ],
    //mode: 'development',
    mode: 'production',
    experiments: {
        syncWebAssembly: true
    },
    module: {
        rules: [
            { test: /\.png$/i, use: 'file-loader' }
        ]
    }
};
