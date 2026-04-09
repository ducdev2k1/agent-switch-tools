import { Button } from '@/components/ui/button'
import { Input } from '@/components/ui/input'
import { Label } from '@/components/ui/label'
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from '@/components/ui/select'
import { Switch } from '@/components/ui/switch'
import { useWebhookConfig } from '@/hooks/use-webhook-config'
import type { WebhookConfig, WebhookTriggerMode } from '@/lib/types'
import { invoke } from '@tauri-apps/api/core'
import {
  AlertTriangle,
  Check,
  ChevronDown,
  ChevronRight,
  Copy,
  Eye,
  EyeOff,
  Send,
  Zap,
} from 'lucide-react'
import { useCallback, useRef, useState } from 'react'
import { useTranslation } from 'react-i18next'
import { toast } from 'sonner'

/** Validate URL: must be https:// or http://localhost */
function isValidWebhookUrl(url: string): boolean {
  if (!url) return false
  try {
    const parsed = new URL(url)
    if (parsed.protocol === 'https:') return true
    if (
      parsed.protocol === 'http:' &&
      (parsed.hostname === 'localhost' || parsed.hostname === '127.0.0.1')
    )
      return true
    return false
  } catch {
    return false
  }
}

export function WebhookSettingsPanel() {
  const { t } = useTranslation()
  const { config, loading, save } = useWebhookConfig()

  // Local form state (edit before saving)
  const [form, setForm] = useState<WebhookConfig | null>(null)
  const draft = form ?? config

  // Secret visibility toggle
  const [showSecret, setShowSecret] = useState(false)

  // Action states
  const [testing, setTesting] = useState(false)
  const [sending, setSending] = useState(false)
  const lastSentRef = useRef<number>(0)

  const COOLDOWN_MS = 30_000

  const updateField = <K extends keyof WebhookConfig>(
    key: K,
    value: WebhookConfig[K],
  ) => {
    setForm((prev) => ({ ...(prev ?? config), [key]: value }))
  }

  const handleSave = useCallback(async () => {
    if (draft.enabled) {
      if (!draft.memberEmail.trim()) {
        toast.error(t('settings.webhook.member_email_required'))
        return
      }
      if (!isValidWebhookUrl(draft.url)) {
        toast.error(t('settings.webhook.url_hint'))
        return
      }
    }
    await save(draft)
    setForm(null)
    toast.success(t('settings.webhook.saved'))
  }, [draft, save, t])

  const handleTest = useCallback(async () => {
    if (!isValidWebhookUrl(draft.url)) {
      toast.error(t('settings.webhook.url_hint'))
      return
    }
    setTesting(true)
    try {
      const res = await invoke<{ success: boolean; message: string }>(
        'send_webhook',
        {
          url: draft.url,
          secret: draft.secret || null,
          testMode: true,
          includeCredentials: draft.includeCredentials,
          memberEmail: draft.memberEmail || null,
        },
      )
      if (res.success) {
        toast.success(t('settings.webhook.test_success'))
      } else {
        toast.error(t('settings.webhook.test_failed', { error: res.message }))
      }
    } catch (e) {
      toast.error(t('settings.webhook.test_failed', { error: String(e) }))
    } finally {
      setTesting(false)
    }
  }, [draft, t])

  const handleSendNow = useCallback(async () => {
    const now = Date.now()
    const elapsed = now - lastSentRef.current
    if (elapsed < COOLDOWN_MS) {
      const remaining = Math.ceil((COOLDOWN_MS - elapsed) / 1000)
      toast.warning(t('settings.webhook.cooldown', { seconds: remaining }))
      return
    }

    if (!isValidWebhookUrl(draft.url)) {
      toast.error(t('settings.webhook.url_hint'))
      return
    }
    setSending(true)
    try {
      const res = await invoke<{ success: boolean; message: string }>(
        'send_webhook',
        {
          url: draft.url,
          secret: draft.secret || null,
          testMode: false,
          includeCredentials: draft.includeCredentials,
          memberEmail: draft.memberEmail || null,
        },
      )
      lastSentRef.current = Date.now()
      if (res.success) {
        toast.success(t('settings.webhook.send_success'))
      } else {
        toast.error(t('settings.webhook.send_failed', { error: res.message }))
      }
    } catch (e) {
      toast.error(t('settings.webhook.send_failed', { error: String(e) }))
    } finally {
      setSending(false)
    }
  }, [draft, t])

  if (loading) {
    return (
      <div className="text-sm text-muted-foreground">
        {t('common.labels.loading')}
      </div>
    )
  }

  const disabled = !draft.enabled
  const hasChanges = form !== null

  return (
    <div className="space-y-6">
      {/* Enable toggle */}
      <div>
        <h3 className="text-xs font-semibold uppercase tracking-wider text-muted-foreground mb-2 px-1">
          {t('settings.webhook.title')}
        </h3>
        <div className="rounded-lg border bg-card">
          <div className="flex items-center justify-between px-4 py-3">
            <div>
              <Label className="text-sm">{t('settings.webhook.enabled')}</Label>
              <p className="text-xs text-muted-foreground mt-0.5">
                {t('settings.webhook.description')}
              </p>
            </div>
            <Switch
              checked={draft.enabled}
              onCheckedChange={(v) => updateField('enabled', v)}
            />
          </div>
        </div>
      </div>

      {/* PII warning */}
      {draft.enabled && (
        <div className="flex items-start gap-2 rounded-lg border border-amber-500/30 bg-amber-500/5 px-4 py-3 text-xs text-amber-700 dark:text-amber-400">
          <AlertTriangle className="size-4 shrink-0 mt-0.5" />
          {t('settings.webhook.pii_warning')}
        </div>
      )}

      {/* Member Email */}
      <div>
        <h3 className="text-xs font-semibold uppercase tracking-wider text-muted-foreground mb-2 px-1">
          {t('settings.webhook.member_email')}
          <span className="text-red-500 ml-0.5">*</span>
        </h3>
        <div className="rounded-lg border bg-card">
          <div className="px-4 py-3 space-y-2">
            <Input
              value={draft.memberEmail}
              onChange={(e) => updateField('memberEmail', e.target.value)}
              placeholder={t('settings.webhook.member_email_placeholder')}
              disabled={disabled}
              className="h-8 text-sm"
            />
            <p className="text-[11px] text-muted-foreground">
              {t('settings.webhook.member_email_hint')}
            </p>
          </div>
        </div>
      </div>

      {/* Configuration fields */}
      <div>
        <h3 className="text-xs font-semibold uppercase tracking-wider text-muted-foreground mb-2 px-1">
          {t('settings.webhook.url')}
          <span className="text-red-500 ml-0.5">*</span>
        </h3>
        <div className="rounded-lg border bg-card">
          {/* Endpoint URL */}
          <div className="px-4 py-3 space-y-2">
            <Input
              value={draft.url}
              onChange={(e) => updateField('url', e.target.value)}
              placeholder={t('settings.webhook.url_placeholder')}
              disabled={disabled}
              className="h-8 text-sm"
            />
            <p className="text-[11px] text-muted-foreground">
              {t('settings.webhook.url_hint')}
            </p>
          </div>

          <div className="border-t" />

          {/* Auth Secret */}
          <div className="px-4 py-3 space-y-1.5">
            <Label className="text-xs text-muted-foreground">
              {t('settings.webhook.secret')}
            </Label>
            <div className="relative">
              <Input
                type={showSecret ? 'text' : 'password'}
                value={draft.secret}
                onChange={(e) => updateField('secret', e.target.value)}
                placeholder={t('settings.webhook.secret_placeholder')}
                disabled={disabled}
                className="pr-10 h-8 text-sm"
              />
              <button
                type="button"
                onClick={() => setShowSecret((v) => !v)}
                disabled={disabled}
                className="absolute right-2 top-1/2 -translate-y-1/2 text-muted-foreground hover:text-foreground disabled:opacity-50 cursor-pointer"
              >
                {showSecret ? (
                  <EyeOff className="size-3.5" />
                ) : (
                  <Eye className="size-3.5" />
                )}
              </button>
            </div>
          </div>

          <div className="border-t" />

          {/* Trigger Mode */}
          <div className="flex items-center justify-between px-4 py-3">
            <Label className="text-sm">
              {t('settings.webhook.trigger_mode')}
            </Label>
            <Select
              value={draft.triggerMode}
              onValueChange={(v) =>
                updateField('triggerMode', v as WebhookTriggerMode)
              }
              disabled={disabled}
            >
              <SelectTrigger className="w-48 h-8 text-xs">
                <SelectValue />
              </SelectTrigger>
              <SelectContent>
                <SelectItem value="manual">
                  {t('settings.webhook.trigger_manual')}
                </SelectItem>
                <SelectItem value="on_startup">
                  {t('settings.webhook.trigger_startup')}
                </SelectItem>
                <SelectItem value="on_change">
                  {t('settings.webhook.trigger_on_change')}
                </SelectItem>
              </SelectContent>
            </Select>
          </div>

          <div className="border-t" />

          {/* Include Credentials */}
          <div className="flex items-center justify-between px-4 py-3">
            <div>
              <Label className="text-sm">
                {t('settings.webhook.include_credentials')}
              </Label>
              <p className="text-xs text-muted-foreground mt-0.5">
                {t('settings.webhook.include_credentials_desc')}
              </p>
            </div>
            <Switch
              checked={draft.includeCredentials}
              onCheckedChange={(v) => updateField('includeCredentials', v)}
              disabled={disabled}
            />
          </div>
        </div>
      </div>

      {/* Sample Payload */}
      <SamplePayload includeCredentials={draft.includeCredentials} />

      {/* Actions */}
      <div className="flex items-center gap-2 flex-wrap">
        <Button
          variant="outline"
          size="sm"
          onClick={handleTest}
          disabled={disabled || testing || !draft.url}
        >
          <Zap className="size-3.5" />
          {testing
            ? t('settings.webhook.testing')
            : t('settings.webhook.test_connection')}
        </Button>
        <Button
          variant="outline"
          size="sm"
          onClick={handleSendNow}
          disabled={disabled || sending || !draft.url}
        >
          <Send className="size-3.5" />
          {sending
            ? t('settings.webhook.sending')
            : t('settings.webhook.send_now')}
        </Button>
        <div className="flex-1" />
        <Button
          size="sm"
          onClick={handleSave}
          disabled={!hasChanges}
        >
          {t('settings.webhook.save')}
        </Button>
      </div>
    </div>
  )
}

function buildSamplePayload(includeCredentials: boolean): string {
  const profile: Record<string, unknown> = {
    name: 'user@example.com',
    email: 'user@example.com',
    subscription_type: 'claude_pro',
    rate_limit_tier: 't3',
    is_active: true,
    is_expired: false,
    usage: {
      five_hour: { utilization: 38.0, resets_at: '2026-04-03T19:30:00Z' },
      seven_day: { utilization: 12.0, resets_at: '2026-04-10T00:00:00Z' },
      seven_day_sonnet: {
        utilization: 5.0,
        resets_at: '2026-04-10T00:00:00Z',
      },
    },
  }

  if (includeCredentials) {
    profile.credentials = {
      claudeAiOauth: {
        accessToken: 'oa-****',
        refreshToken: 'or-****',
        expiresAt: 1749000000,
        scopes: ['user:inference', 'user:profile'],
      },
    }
  }

  return JSON.stringify(
    {
      event: 'usage_report',
      timestamp: '2026-04-03T14:30:00Z',
      app_version: '1.0.5',
      member_email: 'name@example.com',
      device_info: {
        device_id: '550e8400-e29b-41d4-a716-446655440000',
        device_name: 'MacBook Pro',
        hostname: 'my-pc',
      },
      system_info: {
        os_name: 'Ubuntu',
        os_version: '24.04',
        hostname: 'my-pc',
        cpu_name: 'AMD Ryzen 7 5800X',
        cpu_cores: 16,
        ram_total_mb: 32768,
        ram_used_mb: 12456,
        arch: 'x86_64',
      },
      data: { profiles: [profile] },
    },
    null,
    2,
  )
}

function CopyButton({ text }: { text: string }) {
  const [copied, setCopied] = useState(false)
  const handleCopy = () => {
    navigator.clipboard.writeText(text)
    setCopied(true)
    setTimeout(() => setCopied(false), 2000)
  }
  return (
    <button
      type="button"
      onClick={handleCopy}
      className="absolute right-2 top-2 p-1.5 rounded-md text-muted-foreground hover:text-foreground hover:bg-muted transition-colors cursor-pointer z-10"
    >
      {copied ? <Check className="size-3.5" /> : <Copy className="size-3.5" />}
    </button>
  )
}

function SamplePayload({
  includeCredentials,
}: {
  includeCredentials: boolean
}) {
  const { t } = useTranslation()
  const [open, setOpen] = useState(false)
  const sample = buildSamplePayload(includeCredentials)

  return (
    <div>
      <button
        type="button"
        onClick={() => setOpen((v) => !v)}
        className="flex items-center gap-1.5 text-xs font-semibold uppercase tracking-wider text-muted-foreground hover:text-foreground transition-colors cursor-pointer px-1"
      >
        {open ? (
          <ChevronDown className="size-3.5" />
        ) : (
          <ChevronRight className="size-3.5" />
        )}
        {t('settings.webhook.sample_payload')}
      </button>
      {open && (
        <div className="relative mt-2">
          <CopyButton text={sample} />
          <pre className="rounded-lg border bg-muted/50 p-4 pr-10 text-[11px] leading-relaxed font-mono overflow-x-auto max-h-80">
            {sample}
          </pre>
        </div>
      )}
    </div>
  )
}
