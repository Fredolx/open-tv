const CopyWebpackPlugin = require('copy-webpack-plugin');
module.exports = {
  /**
   * This is the main entry point for your application, it's the first file
   * that runs in the main process.
   */
  entry: './src/index.js',
  // Put your normal webpack config below here
  module: {
    rules: require('./webpack.rules'),
  },
  plugins: [
		new CopyWebpackPlugin({
			patterns: [
				{
					from: '../ng-open-tv/dist/ng-open-tv',
					noErrorOnMissing: true,
				},
        {
          from: '../../libs',
          to: "../libs"
        }
			],
		}),
	],
};
