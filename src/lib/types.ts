export interface CredentialInfo {
  subscriptionType: string | null
  rateLimitTier: string | null
  expiresAt: number | null
  isExpired: boolean
  expiresInHours: number | null
  scopes: string[]
  organizationUuid: string | null
}

export interface OAuthAccount {
  accountUuid: string | null
  emailAddress: string | null
  organizationUuid: string | null
  hasExtraUsageEnabled: boolean | null
  billingType: string | null
  accountCreatedAt: string | null
  subscriptionCreatedAt: string | null
  displayName: string | null
  organizationRole: string | null
  workspaceRole: unknown
  organizationName: string | null
}

export interface CredentialProfile {
  name: string
  isActive: boolean
  info: CredentialInfo
  oauthAccount: OAuthAccount | null
}

export interface SwitchResult {
  success: boolean
  claudeWasRunning: boolean
  targetWasExpired: boolean
  message: string
}

export interface ClaudeCliState {
  currentModel: string | null
  sessionCount: number
  envFileExists: boolean
  activeKeys: string[]
}

export interface UsageStats {
  totalSessions: number
  recentSessions7d: number
  currentModel: string | null
  hasRestrictions: boolean
}

export interface UsageBucket {
  /** 0-100. Semantics depend on `remainingBased`:
   *  - false/undefined (Claude CLI): % USED — red when high, bar fills with consumption.
   *  - true (Antigravity): % REMAINING — red when low, bar fills opposite direction. */
  utilization: number | null
  resetsAt: string | null
  /** Dynamic label for multi-model providers (e.g. Antigravity). Legacy Claude CLI slots omit this. */
  label?: string | null
  /** When true, `utilization` is remaining % (Antigravity) instead of used % (default). */
  remainingBased?: boolean
}

export interface UsageLimits {
  fiveHour: UsageBucket | null
  sevenDay: UsageBucket | null
  sevenDaySonnet: UsageBucket | null
  /** Dynamic buckets for providers with arbitrary model groupings. When non-empty, frontend renders these instead of the legacy fixed slots. */
  buckets?: UsageBucket[]
}

export interface RefreshResult {
  success: boolean
  message: string
}

/** Token counts split by billing category (Claude Code cost analytics). */
export interface TokenBreakdown {
  input: number
  output: number
  cacheRead: number
  cacheCreation: number
}

export interface DayUsage {
  date: string
  tokens: TokenBreakdown
  costUsd: number | null
}

export interface ModelUsage {
  model: string
  tokens: TokenBreakdown
  costUsd: number | null
}

export interface SessionUsage {
  id: string
  date: string
  model: string
  project: string
  tokens: TokenBreakdown
  costUsd: number | null
}

/** Provenance of cost figures: live fetch, disk cache, or no pricing data. */
export type PriceStatus = 'live' | 'saved' | 'hidden'

/** Per-profile scheduled-priming configuration. */
export interface AutoPrimeSetting {
  enabled: boolean
  /** Local time of day to prime, "HH:MM" (24h). */
  time: string
  lastPrimedDate: string | null
  lastResult: string | null
}

/** Outcome of a single prime attempt (discriminated on `status`). */
export type PrimeResult =
  | { status: 'success'; resetAt: string }
  | { status: 'hold'; resetAt: string }
  | { status: 'failed'; reason: string }
  | { status: 'skipped'; reason: string }

/** Per-day aggregate of prime outcomes. */
export interface PrimeDayStat {
  date: string
  success: number
  failed: number
  hold: number
  skip: number
}

export interface UsageReport {
  total: TokenBreakdown
  totalCostUsd: number | null
  today: TokenBreakdown
  todayCostUsd: number | null
  daily: DayUsage[]
  byModel: ModelUsage[]
  sessions: SessionUsage[]
  generatedAt: string
  priceStatus: PriceStatus
  priceUpdatedAt: string | null
}

export interface ProfileUsage {
  lastActiveAt: string | null
  totalActiveMinutes: number
}

export interface ManagerMeta {
  activeProfileName: string | null
  lastSwitchedAt: string | null
  usageHistory: Record<string, ProfileUsage>
}

export interface DeviceInfo {
  deviceId: string
  deviceName: string
  hostname: string
  createdAt: string
  lastSeenAt: string
}

export interface SystemInfo {
  osName: string
  osVersion: string
  hostname: string
  cpuName: string
  cpuCores: number
  ramTotalMb: number
  ramUsedMb: number
  arch: string
}

export interface AppUpdaterState {
  updateVersion: string | null
  updateBody: string | null
  showModal: boolean
  dismissModal: () => Promise<void>
  installing: boolean
  install: () => Promise<void>
  checking: boolean
  checkForUpdates: () => Promise<string | null>
}

export type WebhookTriggerMode = 'manual' | 'on_startup' | 'on_change'

export interface WebhookConfig {
  enabled: boolean
  url: string
  secret: string
  apiKey: string
  triggerMode: WebhookTriggerMode
  includeCredentials: boolean
  includeSessionUsage: boolean
  memberEmail: string
}

export interface WebhookResponse {
  success: boolean
  statusCode: number | null
  message: string
}

// ========== IDE Multi-Account ==========

export type IdeType =
  | 'cursor'
  | 'antigravity'
  | 'antigravity-ide'
  | 'antigravity-cli'
  | 'windsurf'

/** IDEs without a public single-user quota API (see research report 260422-0921). */
const IDE_QUOTA_UNSUPPORTED: readonly IdeType[] = ['cursor', 'windsurf']

export function isIdeQuotaSupported(ideType: IdeType): boolean {
  return !IDE_QUOTA_UNSUPPORTED.includes(ideType)
}

export interface IdeInfo {
  ideType: IdeType
  displayName: string
  isInstalled: boolean
}

export interface IdeProfile {
  name: string
  isActive: boolean
  email: string | null
  membershipType: string | null
  displayName: string | null
  ideType: IdeType
  usage?: UsageLimits | null
}

export interface IdeSwitchResult {
  success: boolean
  ideWasRunning: boolean
  message: string
}

// ========== Session Usage ==========

export interface SessionUsageSummary {
  sessionId: string
  project: string
  branch: string
  model: string
  startedAt: string
  endedAt: string
  totalInputTokens: number
  totalOutputTokens: number
  totalCacheRead: number
  totalCacheWrite: number
  messageCount: number
}
