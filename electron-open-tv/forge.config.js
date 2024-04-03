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
      name: '@electron-forge/maker-wix',
      config: {
        language: 1033,
        manufacturer: 'Lachapelle Software',
        icon: "resources/icon.ico"
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
        icon: 'resources/icon.png',
        categories: ['AudioVideo'],
        depends: ['mpv']
      },
    },
    {
      name: '@electron-forge/maker-rpm',
      config: {
        icon: 'resources/icon.png',
        categories: ['AudioVideo'],
        requires: ['mpv']
      },
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
