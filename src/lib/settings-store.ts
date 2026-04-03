import { LazyStore } from '@tauri-apps/plugin-store'

/** Shared settings store instance — all hooks should use this singleton */
export const settingsStore = new LazyStore('settings.json')
