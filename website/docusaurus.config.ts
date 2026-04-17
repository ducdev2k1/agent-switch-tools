import {themes as prismThemes} from 'prism-react-renderer';
import type {Config} from '@docusaurus/types';
import type * as Preset from '@docusaurus/preset-classic';

const config: Config = {
  title: 'Agent Switch Tools',
  tagline: 'Manage multiple Claude Code CLI accounts — switch profiles, monitor quota, stay organized.',
  favicon: 'img/favicon.ico',

  future: {
    v4: true,
  },

  url: 'https://ducdev2k1.github.io',
  baseUrl: '/agent-switch-tools/',

  organizationName: 'ducdev2k1',
  projectName: 'agent-switch-tools',

  onBrokenLinks: 'throw',

  i18n: {
    defaultLocale: 'vi',
    locales: ['vi', 'en'],
    localeConfigs: {
      vi: {label: 'Tiếng Việt'},
      en: {label: 'English'},
    },
  },

  presets: [
    [
      'classic',
      {
        docs: {
          sidebarPath: './sidebars.ts',
          editUrl: 'https://github.com/ducdev2k1/agent-switch-tools/tree/main/website/',
        },
        blog: {
          showReadingTime: true,
          blogTitle: 'Release Notes',
          blogDescription: 'Agent Switch Tools release notes and changelog',
          blogSidebarTitle: 'All Releases',
          blogSidebarCount: 'ALL',
          feedOptions: {
            type: ['rss', 'atom'],
            xslt: true,
          },
          editUrl: 'https://github.com/ducdev2k1/agent-switch-tools/tree/main/website/',
          onInlineTags: 'warn',
          onInlineAuthors: 'warn',
          onUntruncatedBlogPosts: 'ignore',
        },
        theme: {
          customCss: './src/css/custom.css',
        },
      } satisfies Preset.Options,
    ],
  ],

  themeConfig: {
    image: 'img/logo.png',
    colorMode: {
      respectPrefersColorScheme: true,
    },
    navbar: {
      title: 'Agent Switch Tools',
      logo: {
        alt: 'Agent Switch Tools Logo',
        src: 'img/logo.png',
      },
      items: [
        {
          type: 'docSidebar',
          sidebarId: 'docsSidebar',
          position: 'left',
          label: 'Docs',
        },
        {to: '/blog', label: 'Release Notes', position: 'left'},
        {
          type: 'localeDropdown',
          position: 'right',
        },
        {
          href: 'https://github.com/ducdev2k1/agent-switch-tools',
          label: 'GitHub',
          position: 'right',
        },
      ],
    },
    footer: {
      style: 'dark',
      links: [
        {
          title: 'Docs',
          items: [
            {label: 'Tổng quan', to: '/docs/tong-quan'},
            {label: 'Tính năng', to: '/docs/tinh-nang'},
            {label: 'Hướng dẫn sử dụng', to: '/docs/huong-dan-su-dung'},
          ],
        },
        {
          title: 'Release Notes',
          items: [
            {label: 'Tất cả phiên bản', to: '/blog'},
          ],
        },
        {
          title: 'Links',
          items: [
            {
              label: 'GitHub',
              href: 'https://github.com/ducdev2k1/agent-switch-tools',
            },
            {
              label: 'Download',
              href: 'https://github.com/ducdev2k1/agent-switch-tools/releases/latest',
            },
            {
              label: 'Issues',
              href: 'https://github.com/ducdev2k1/agent-switch-tools/issues',
            },
          ],
        },
      ],
      copyright: `Copyright © ${new Date().getFullYear()} ducdev2k1. Built with Docusaurus.`,
    },
    prism: {
      theme: prismThemes.github,
      darkTheme: prismThemes.dracula,
      additionalLanguages: ['rust', 'bash', 'json'],
    },
  } satisfies Preset.ThemeConfig,
};

export default config;
