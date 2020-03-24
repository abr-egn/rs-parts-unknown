const path = require('path');
const webpack = require('webpack');
const WasmPackPlugin = require("@wasm-tool/wasm-pack-plugin");
const CopyWebpackPlugin = require("copy-webpack-plugin");

module.exports = {
    entry: './ts/index.ts',
    devtool: 'inline-source-map',
    module: {
        rules: [
            {
                test: /\.tsx?$/,
                use: 'ts-loader',
                exclude: /node_modules/,
            }
        ],
    },
    resolve: {
        extensions: [ '.tsx', '.ts', '.js' ],
    },
    output: {
        path: path.resolve(__dirname, 'dist'),
        filename: 'bundle.js',
    },
    plugins: [
        new WasmPackPlugin({
            crateDirectory: path.resolve(__dirname, ".."),
            outDir: path.resolve(__dirname, "wasm"),
            withTypeScript: true
        }),
        new CopyWebpackPlugin([
            { from: 'static' }
        ])
    ],
    mode: 'development'
};