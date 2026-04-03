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
  utilization: number | null
  resetsAt: string | null
}

export interface UsageLimits {
  fiveHour: UsageBucket | null
  sevenDay: UsageBucket | null
  sevenDaySonnet: UsageBucket | null
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

export type WebhookTriggerMode = 'manual' | 'on_startup' | 'on_change'

export interface WebhookConfig {
  enabled: boolean
  url: string
  secret: string
  triggerMode: WebhookTriggerMode
}

export interface WebhookResponse {
  success: boolean
  statusCode: number | null
  message: string
}
