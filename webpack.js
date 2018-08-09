const path = require('path');
const nodeExternals = require('webpack-node-externals');

module.exports = {
  devtool: 'source-map',
  entry: './src/index.ts',
  externals: [ nodeExternals() ],
  mode: process.env.WEBPACK_MODE || 'development',
  module: {
    rules: [
      {
        test: /\.ts$/,
        use: [
          { loader: 'ts-loader' }
        ],
        exclude: /node_modules/
      }
    ]
  },
  resolve: {
    extensions: [ '.js', '.ts' ]
  },
  target: 'node',
  output: {
    filename: 'index.js',
    sourceMapFilename: 'index.map.js',
    libraryTarget: 'commonjs',
    path: path.join(__dirname, 'lib'),
  }
};
