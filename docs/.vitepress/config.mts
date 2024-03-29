import { defineConfig } from 'vitepress'

// https://vitepress.dev/reference/site-config
export default defineConfig({
  title: "Astria",
  description: "The Shared Sequencer Network",
  themeConfig: {
    logo: { src: '/astria-logo-mini.svg', width: 24, height: 24 },

    // https://vitepress.dev/reference/default-theme-config
    nav: [
      { text: 'Home', link: '/' },
      { text: 'Just Rollup', link: '/Introduction' }
    ],

    sidebar: [
      {
        text: 'Info',
        items: [
          { text: 'Introduction', link: '/overview/introduction' },
          { text: 'Runtime API Examples', link: '/api-examples' }
        ]
      }
    ],

    socialLinks: [
      { icon: 'github', link: 'https://github.com/vuejs/vitepress' }
    ]
  }
})
