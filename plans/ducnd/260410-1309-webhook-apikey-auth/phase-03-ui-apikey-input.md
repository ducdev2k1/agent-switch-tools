# Phase 03: UI — Add API Key input to webhook settings panel

## Context
- Parent plan: [plan.md](plan.md)
- Depends on: Phase 02 (TypeScript types must have `apiKey` field)

## Overview
- Priority: P1
- Status: pending
- Add API Key input field to webhook settings UI, placed above Auth Secret

## Key Insights
- UI at `src/components/webhook-settings-panel.tsx`
- Auth Secret section at lines 244-271 (with show/hide toggle)
- API Key field should follow same masked input pattern as secret
- Place API Key BEFORE Auth Secret since it's the primary auth for target service

## Requirements
- Add API Key masked input field with show/hide toggle
- Place between Endpoint URL section and Auth Secret section
- Reuse same pattern as secret field (password input + eye toggle)
- Use i18n keys for labels

## Related Code Files
- Modify: `src/components/webhook-settings-panel.tsx` (insert after URL section, before Auth Secret)

## Implementation Steps

1. Add `showApiKey` state alongside existing `showSecret` state (line 56):
   ```tsx
   const [showApiKey, setShowApiKey] = useState(false)
   ```

2. Insert API Key section after Endpoint URL section (after line 240, before `<div className="border-t" />`):
   ```tsx
   <div className="border-t" />
   
   {/* API Key */}
   <div className="px-4 py-3 space-y-1.5">
     <Label className="text-xs text-muted-foreground">
       {t('settings.webhook.api_key')}
     </Label>
     <div className="relative">
       <Input
         type={showApiKey ? 'text' : 'password'}
         value={draft.apiKey}
         onChange={(e) => updateField('apiKey', e.target.value)}
         placeholder={t('settings.webhook.api_key_placeholder')}
         disabled={disabled}
         className="pr-10 h-8 text-sm"
       />
       <button
         type="button"
         onClick={() => setShowApiKey((v) => !v)}
         disabled={disabled}
         className="absolute right-2 top-1/2 -translate-y-1/2 text-muted-foreground hover:text-foreground disabled:opacity-50 cursor-pointer"
       >
         {showApiKey ? <EyeOff className="size-3.5" /> : <Eye className="size-3.5" />}
       </button>
     </div>
     <p className="text-[11px] text-muted-foreground">
       {t('settings.webhook.api_key_hint')}
     </p>
   </div>
   ```

## Todo
- [ ] Add `showApiKey` state
- [ ] Add API Key input section in UI
- [ ] Verify UI renders correctly

## Success Criteria
- API Key field visible in webhook settings
- Masked by default with show/hide toggle
- Value persists through save/load cycle
