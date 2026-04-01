export interface CredentialInfo {
  subscriptionType: string | null
  rateLimitTier: string | null
  expiresAt: number | null
  scopes: string[]
  organizationUuid: string | null
}

export interface CredentialProfile {
  name: string
  isActive: boolean
  info: CredentialInfo
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
