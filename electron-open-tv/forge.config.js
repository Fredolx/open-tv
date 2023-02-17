module.exports = {
  packagerConfig: {
    icon: "resources/icon",
    executableName: "open-tv"
  },
  rebuildConfig: {},
  makers: [
    {
      name: '@electron-forge/maker-squirrel',
      config: {
        authors: 'Frédéric Lachapelle',
        description: "IPTV app based on Electron and Angular",
        setupIcon: 'resources/icon.ico'
      },
    },
    {
      name: '@electron-forge/maker-appx',
      config: {
        publisher: 'CN=D7F3F5FA-82AF-4A01-ACE9-073CE4F7FFA9',
      }
    },
    {
      name: '@electron-forge/maker-zip',
      platforms: ['darwin'],
    },
    {
      name: '@electron-forge/maker-dmg',
      config: {
        icon: 'resources/icon.icns'
      }
    },
    {
      name: '@electron-forge/maker-deb',
      bin: "Open TV",
      config: {
        icon: 'resources/icon.png'
      },
    },
    {
      name: '@electron-forge/maker-rpm',
      config: {},
    },
  ],
  plugins: [
    {
      name: '@electron-forge/plugin-webpack',
      config: {
        mainConfig: './webpack.main.config.js',
        renderer: {
          config: './webpack.renderer.config.js',
          entryPoints: [
            {
              name: 'main_window',
              preload: {
                js: './src/preload.js',
              },
            },
          ],
        },
      },
    },
  ],
};
