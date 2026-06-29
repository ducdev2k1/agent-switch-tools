import { Button } from '@/components/ui/button'
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card'
import { Input } from '@/components/ui/input'
import { useAutoPrime } from '@/hooks/use-auto-prime'
import { useCredentialProfiles } from '@/hooks/use-profiles'
import type { PrimeResult } from '@/lib/types'
import { useState } from 'react'
import { useTranslation } from 'react-i18next'
import { toast } from 'sonner'
import { AutoPrimeRow } from './auto-prime-row'
import { AutoPrimeStats } from './auto-prime-stats'

const TIME_RE = /^([01]\d|2[0-3]):[0-5]\d$/

export function AutoSessionView() {
  const { t } = useTranslation()
  const { profiles } = useCredentialProfiles()
  const { settings, log, stats, setAutoPrime, setAll, primeNow } = useAutoPrime()
  const [allTime, setAllTime] = useState('09:00')

  const names = profiles.map((p) => p.name)
  const validAll = TIME_RE.test(allTime)

  const describe = (r: PrimeResult): string => {
    if (r.status === 'success')
      return t('auto_session.result.success', { reset: r.resetAt })
    if (r.status === 'hold')
      return t('auto_session.result.hold', { reset: r.resetAt })
    if (r.status === 'failed')
      return t('auto_session.result.failed', { reason: r.reason })
    return t('auto_session.result.skipped', { reason: r.reason })
  }

  const handlePrimeNow = async (name: string) => {
    const result = await primeNow(name)
    const msg = describe(result)
    if (result.status === 'success') toast.success(msg)
    else if (result.status === 'failed') toast.error(msg)
    else toast.info(msg)
    return result
  }

  const applyAll = (enabled: boolean) => {
    if (!validAll || names.length === 0) return
    setAll(names, enabled, allTime)
    toast.success(
      enabled
        ? t('auto_session.enabled_toast', { count: names.length })
        : t('auto_session.disabled_toast'),
    )
  }

  return (
    <div className="space-y-6">
      <div className="rounded-lg border border-warning/30 bg-warning/5 px-4 py-3 text-sm">
        {t('auto_session.banner')}
      </div>

      <Card>
        <CardHeader>
          <CardTitle className="text-sm">
            {t('auto_session.apply_all')}
          </CardTitle>
        </CardHeader>
        <CardContent className="flex items-center gap-3">
          <Input
            type="time"
            value={allTime}
            onChange={(e) => setAllTime(e.target.value)}
            className="w-28"
          />
          <Button
            size="sm"
            onClick={() => applyAll(true)}
            disabled={!validAll || names.length === 0}
          >
            {t('auto_session.enable_all')}
          </Button>
          <Button
            size="sm"
            variant="outline"
            onClick={() => applyAll(false)}
            disabled={names.length === 0}
          >
            {t('auto_session.disable_all')}
          </Button>
        </CardContent>
      </Card>

      <div className="space-y-2">
        {names.length === 0 ? (
          <p className="py-4 text-center text-sm text-muted-foreground">
            {t('auto_session.no_profiles')}
          </p>
        ) : (
          names.map((name) => (
            <AutoPrimeRow
              // Include the setting signature so the row re-initializes once
              // async settings arrive after the profiles already mounted.
              key={`${name}:${settings[name]?.time ?? ''}:${settings[name]?.enabled ?? ''}`}
              name={name}
              setting={settings[name]}
              onSave={setAutoPrime}
              onPrimeNow={handlePrimeNow}
            />
          ))
        )}
      </div>

      <Card>
        <CardHeader>
          <CardTitle className="text-sm">
            {t('auto_session.daily_stats')}
          </CardTitle>
        </CardHeader>
        <CardContent>
          <AutoPrimeStats stats={stats} />
        </CardContent>
      </Card>

      <Card>
        <CardHeader>
          <CardTitle className="text-sm">
            {t('auto_session.activity_log')}
          </CardTitle>
        </CardHeader>
        <CardContent>
          {log ? (
            <pre className="max-h-48 overflow-auto whitespace-pre-wrap rounded bg-muted/40 p-3 text-xs">
              {log}
            </pre>
          ) : (
            <p className="text-sm text-muted-foreground">
              {t('auto_session.no_activity')}
            </p>
          )}
        </CardContent>
      </Card>
    </div>
  )
}
