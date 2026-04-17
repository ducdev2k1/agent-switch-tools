# Deployment Guide

## CI/CD Overview

Two GitHub Actions workflows automate building and releasing Agent Switch Tools.

| Workflow                                      | Trigger           | Purpose                       |
| --------------------------------------------- | ----------------- | ----------------------------- |
| **CI** (`.github/workflows/ci.yml`)           | Push/PR to `main` | Build check on all platforms  |
| **Release** (`.github/workflows/release.yml`) | Push tag `v*`     | Build + create GitHub Release |

### Supported Platforms

| Platform            | Runner           | Target  | Artifacts             |
| ------------------- | ---------------- | ------- | --------------------- |
| Linux (22.04+)      | `ubuntu-22.04`   | x86_64  | `.deb`, `.AppImage`   |
| Linux (24.04+)      | `ubuntu-latest`  | x86_64  | `.deb`, `.AppImage`   |
| macOS Intel         | `macos-13`       | x86_64  | `.dmg`                |
| macOS Apple Silicon | `macos-latest`   | aarch64 | `.dmg`                |
| Windows 10+         | `windows-latest` | x86_64  | `.msi`, `.exe` (NSIS) |

## Release Process

### 1. Update Version

Edit version in both files (must match):

- `package.json` → `"version": "X.Y.Z"`
- `src-tauri/tauri.conf.json` → `"version": "X.Y.Z"`

### 2. Commit & Tag

```bash
git add package.json src-tauri/tauri.conf.json
git commit -m "chore: bump version to X.Y.Z"
git tag vX.Y.Z
git push origin main --tags
```

### 3. Review & Publish

1. GitHub Actions builds on all 5 platform targets (~15-20 min)
2. A **draft release** is created at `https://github.com/<owner>/agent-switch-tools/releases`
3. Review the draft, edit release notes if needed
4. Click **Publish release**

## Linux System Dependencies

CI installs these automatically. For local development on Ubuntu/Debian:

```bash
sudo apt-get install -y \
  libwebkit2gtk-4.1-dev \
  librsvg2-dev \
  patchelf \
  libssl-dev \
  libgtk-3-dev \
  libayatana-appindicator3-dev
```

## Troubleshooting

### Build fails on Linux

- Missing system deps → check `apt-get install` list above
- WebKit version mismatch → ensure `libwebkit2gtk-4.1-dev` (not 4.0)

### Build fails on macOS

- Xcode CLT missing → runner has it pre-installed, but locally run `xcode-select --install`
- Wrong target → workflow builds both `x86_64-apple-darwin` (Intel) and `aarch64-apple-darwin` (Apple Silicon)

### Build fails on Windows

- Long path issues → enable long paths: `git config --system core.longpaths true`
- MSVC missing → runner has it pre-installed, locally install Visual Studio Build Tools

### Release not created

- Tag format must be `vX.Y.Z` (e.g., `v0.1.0`)
- Version in `tauri.conf.json` must match tag (without `v` prefix)
- Check Actions tab for workflow run status

## Windows SmartScreen

Windows SmartScreen blocks unsigned binaries with zero reputation. Each new release resets reputation since the file hash changes.

**Current mitigation:**
- `publisher` and `copyright` metadata in `tauri.conf.json`
- Install guide in README + release notes with "More info → Run anyway" steps
- `.msi` installer as alternative (better trusted by Windows)

## Code Signing

Currently unsigned. Users may see OS warnings on first launch.

### Windows — SignPath Foundation (FREE for OSS)

Release workflow (`release.yml`) has a separate `release-windows` job with commented SignPath integration steps. To enable:

1. Register project at https://signpath.org (free for open-source)
2. Add secrets to GitHub repo:
   - `SIGNPATH_API_TOKEN`
   - `SIGNPATH_ORG_ID`
   - `SIGNPATH_PROJECT_SLUG`
   - `SIGNPATH_POLICY_SLUG`
3. Uncomment SignPath steps in `release.yml`, remove `tagName` from Windows build step

**Fallback options:**
- Azure Trusted Signing ($9.99/mo) — immediate SmartScreen bypass
- Certum Open Source (25-49€/yr) — cheap OV certificate

### macOS (requires Apple Developer account)

Add these secrets to GitHub repo:

- `APPLE_CERTIFICATE` — Base64-encoded .p12 certificate
- `APPLE_CERTIFICATE_PASSWORD` — Certificate password
- `APPLE_SIGNING_IDENTITY` — Certificate identity
- `APPLE_ID` — Apple ID email
- `APPLE_PASSWORD` — App-specific password
- `APPLE_TEAM_ID` — Team ID
