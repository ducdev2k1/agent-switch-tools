export interface CredentialInfo {
  subscriptionType: string | null
  rateLimitTier: string | null
  expiresAt: number | null
  isExpired: boolean
  expiresInHours: number | null
  scopes: string[]
  organizationUuid: string | null
}

export interface CredentialProfile {
  name: string
  isActive: boolean
  info: CredentialInfo
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

export interface ProfileUsage {
  lastActiveAt: string | null
  totalActiveMinutes: number
}

export interface ManagerMeta {
  activeProfileName: string | null
  lastSwitchedAt: string | null
  usageHistory: Record<string, ProfileUsage>
}
