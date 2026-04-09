import type {ReactNode} from 'react';
import clsx from 'clsx';
import Link from '@docusaurus/Link';
import useDocusaurusContext from '@docusaurus/useDocusaurusContext';
import Layout from '@theme/Layout';
import Heading from '@theme/Heading';

import styles from './index.module.css';

type FeatureItem = {
  title: string;
  emoji: string;
  description: string;
};

const features: FeatureItem[] = [
  {
    title: 'Multi-Account Management',
    emoji: '👥',
    description: 'Save, switch, and manage multiple Claude Code CLI accounts with one click. No more logout/login cycles.',
  },
  {
    title: 'Smart Quota Monitoring',
    emoji: '📊',
    description: 'Real-time usage tracking for all profiles — 5h, 7d, and Sonnet limits with auto-refresh every 5 minutes.',
  },
  {
    title: 'System Tray Integration',
    emoji: '⚡',
    description: 'Switch accounts instantly from the system tray without opening the app window.',
  },
  {
    title: '100% Local & Secure',
    emoji: '🔒',
    description: 'Zero telemetry. All data stays on your machine. Only connects to official Anthropic API for quota.',
  },
  {
    title: 'Auto Token Refresh',
    emoji: '🔄',
    description: 'Expired tokens are refreshed automatically in the background. One click to revive any expired profile.',
  },
  {
    title: 'Webhook Reports',
    emoji: '📡',
    description: 'Send usage reports to external endpoints — perfect for team leads monitoring quota across devices.',
  },
];

function HomepageHeader(): ReactNode {
  const {siteConfig} = useDocusaurusContext();
  return (
    <header className={clsx('hero hero--primary', styles.heroBanner)}>
      <div className="container">
        <img
          src="img/logo.png"
          alt="Claude Tools Logo"
          className={styles.heroLogo}
        />
        <Heading as="h1" className="hero__title">
          {siteConfig.title}
        </Heading>
        <p className="hero__subtitle">{siteConfig.tagline}</p>
        <div className={styles.buttons}>
          <Link
            className="button button--secondary button--lg"
            href="https://github.com/ducdev2k1/claude-tools/releases/latest">
            Download Latest
          </Link>
          <Link
            className="button button--outline button--lg"
            style={{color: '#fafafa', borderColor: '#fafafa', marginLeft: '1rem'}}
            to="/docs/tong-quan">
            Documentation
          </Link>
        </div>
      </div>
    </header>
  );
}

function Feature({title, emoji, description}: FeatureItem): ReactNode {
  return (
    <div className={clsx('col col--4')}>
      <div className="text--center padding-horiz--md" style={{marginBottom: '2rem'}}>
        <div style={{fontSize: '3rem', marginBottom: '0.5rem'}}>{emoji}</div>
        <Heading as="h3">{title}</Heading>
        <p>{description}</p>
      </div>
    </div>
  );
}

function TechStack(): ReactNode {
  return (
    <section className={styles.techStack}>
      <div className="container">
        <Heading as="h2" className="text--center" style={{marginBottom: '2rem'}}>
          Tech Stack
        </Heading>
        <div className="row" style={{justifyContent: 'center', gap: '1rem', flexWrap: 'wrap'}}>
          {['Tauri v2', 'React 19', 'TypeScript', 'Rust', 'Tailwind CSS', 'Vite 7'].map(
            (tech) => (
              <span key={tech} className={styles.techBadge}>
                {tech}
              </span>
            ),
          )}
        </div>
      </div>
    </section>
  );
}

function Platforms(): ReactNode {
  const platforms = [
    {name: 'Windows', file: '.msi / .exe'},
    {name: 'macOS', file: '.dmg'},
    {name: 'Linux', file: '.deb / .AppImage'},
  ];
  return (
    <section style={{padding: '2rem 0'}}>
      <div className="container">
        <Heading as="h2" className="text--center" style={{marginBottom: '2rem'}}>
          Cross-Platform
        </Heading>
        <div className="row" style={{justifyContent: 'center'}}>
          {platforms.map(({name, file}) => (
            <div key={name} className="col col--3 text--center" style={{marginBottom: '1rem'}}>
              <Heading as="h4">{name}</Heading>
              <code>{file}</code>
            </div>
          ))}
        </div>
      </div>
    </section>
  );
}

export default function Home(): ReactNode {
  const {siteConfig} = useDocusaurusContext();
  return (
    <Layout title="Home" description={siteConfig.tagline}>
      <HomepageHeader />
      <main>
        <section style={{padding: '2rem 0'}}>
          <div className="container">
            <div className="row">
              {features.map((props, idx) => (
                <Feature key={idx} {...props} />
              ))}
            </div>
          </div>
        </section>
        <TechStack />
        <Platforms />
      </main>
    </Layout>
  );
}
