# Code Review — usage-limits-display 12h clock + unsupported prop

**Date:** 2026-04-22
**Scope:** `src/components/usage-limits-display.tsx`, `src/components/ide-profile-card.tsx`, `src/components/ide-profile-table.tsx`, `src/locales/{vi,en}.json`

## Summary

Changes are small, focused, well-scoped. `formatResetsAt` correctly handles null/invalid. `unsupported` branch short-circuits before loading/null checks — correct order. i18n keys follow existing `usage_*` convention. No security, perf, or hook-order issues.

**Overall score: 8.5 / 10**

---

## Critical Issues (must-fix)

None.

---

## Minor Issues (nice-to-have)

### 1. `hour12: true` is ignored in some locales (vi-VN default formatter)
`toLocaleTimeString(undefined, { hour12: true })` — `undefined` = runtime default locale. When user locale is `vi-VN`, Intl actually *does* honor `hour12: true` in modern V8/Chromium (Tauri uses webview), so rendering `"3:45 CH"` (CH = chiều = PM) is expected. But:
- On some locales the output ends up as `"3:45 SA"` / `"3:45 CH"` (localized am/pm markers), not English "PM". User spec said "like `(3:45 PM)`" — will not be literal PM in vi.
- Recommendation: if you want strict English AM/PM, pass `'en-US'` explicitly: `d.toLocaleTimeString('en-US', {...})`. Or accept localized output (probably fine — it's already locale-aware UI).
- **Not blocking**, but worth confirming intended behavior.

### 2. `R:` line collapses to empty when resetting
In `UsageRow`: `resetText.replace(/[^0-9hm]/g, '').trim()`. When bucket hits reset window, `formatResetsIn` returns `t('common.labels.usage_resetting')` = `"Resetting..."` / `"Đang đặt lại..."`. Strip regex filters out all letters/punctuation → empty string. Result: UI shows `R: ` (nothing) plus `(3:45 PM)` in parentheses. Looks broken.
- Fix: check `if (resetText && strippedText)` or render `usage_resetting` verbatim without regex strip.
- This is a **pre-existing bug** exposed more visibly now that `(absText)` is added beside it — empty `R:` with a time in parens is more confusing than empty `R:` alone.

### 3. Missing `title` / tooltip on unsupported message
Italic muted grey at `text-[10px] text-muted-foreground/60` is low-contrast. Screen readers get the text but there's no hover explanation of *why* it's unavailable (e.g. "no public quota API for Windsurf/Cursor").
- Nice-to-have: add `title={t('common.labels.usage_unsupported_tooltip')}` with longer explanation. Non-blocking for this scope.

### 4. `unsupported` + `compact` = null — arguably wrong
`compact` mode appears in tray/menu (per codebase pattern). Returning `null` silently hides the row entirely in compact contexts. Caller may expect *something* (e.g. a `—` dash) so user can tell the profile exists but quota data is unavailable vs profile has zero usage.
- Current behavior is defensible (keep compact line clean) but document intent. If compact mode is used somewhere that shows multiple profiles side-by-side, the visual alignment may suffer.
- Recommendation: document this in a code comment next to `if (compact) return null`.

### 5. Duplicate unsupported check in both card + table
`ideType === 'windsurf' || ideType === 'cursor'` is repeated in `ide-profile-card.tsx:85` and `ide-profile-table.tsx:122`. If Cursor quota API is later discovered or a third IDE joins, you'll need to update two spots.
- DRY fix: extract `const QUOTA_UNSUPPORTED_IDES = new Set(['windsurf', 'cursor'])` or helper `isQuotaUnsupported(ideType)` in a shared module (maybe next to `useIdeUsage`).

### 6. `useIdeUsage` still fetches for unsupported IDEs?
Not verified in this review — but if `useIdeUsage('windsurf', ...)` still triggers a network call / Tauri invoke, that's wasted work since UI ignores result. Worth checking the hook bails early for unsupported IDEs.
- **Unresolved — see questions below.**

---

## Edge Case Coverage Assessment

| Case | Covered? | Notes |
|------|----------|-------|
| `resetsAt = null` | Yes | `formatResetsAt` returns null, conditional render skips `(absText)` |
| `resetsAt` invalid ISO | Yes | `isNaN(d.getTime())` check |
| `resetsAt` in past (diff ≤ 0) | Partial | Shows "Resetting..." in full `resetText` but regex strip empties `R:` line (issue #2) |
| `unsupported=true` + `loading=true` | Yes | `unsupported` checked first, short-circuits |
| `unsupported=true` + `limits != null` | Yes | Same short-circuit |
| `unsupported=true` + `compact=true` | Yes | Returns null (intentional — issue #4) |
| Locale without AM/PM concept | Yes | Intl.DateTimeFormat handles gracefully; output will use locale's time markers |
| DST transition around resetsAt | Yes (via Intl) | Browser Intl handles DST correctly |
| Extremely long bucket (>24h) | OK | `formatResetsIn` returns hours; `formatResetsAt` shows clock time only (no date) — could be misleading if reset is *tomorrow* 3:45 PM, shows just "(3:45 PM)" |

**Edge case #9 (long reset window showing ambiguous time)** is worth a thought — if reset is in 23h, `R: 23h (3:45 PM)` could mean today or tomorrow. Low priority for 5h/7d buckets since "7 day" already implies future date ambiguity.

---

## Positive Observations

- `formatResetsAt` is a pure function, easy to unit test.
- `unsupported` branch placed at top of render — correct precedence (checked before loading, before null limits).
- No new hooks or state added — zero re-render cost impact.
- i18n key `usage_unsupported` matches existing `usage_*` naming. Both `vi` and `en` updated.
- Prop is optional (`unsupported?: boolean`), backwards-compatible — existing `claude-code` callers unchanged.
- No console.log / debug leftovers.

---

## Recommended Actions

1. **(Minor)** Fix `R:` empty-string regex behavior when `resetText === usage_resetting` — render raw text instead of stripping.
2. **(Minor)** Extract `isQuotaUnsupported(ideType)` helper to DRY up card + table callers.
3. **(Nice)** Add brief code comment explaining `compact + unsupported → null` intent.
4. **(Nice)** Verify `useIdeUsage` skips fetch for unsupported IDEs (see unresolved).
5. **(Optional)** Force `en-US` locale for `formatResetsAt` if strict "3:45 PM" English format is desired; otherwise accept locale-native output.

---

## Unresolved Questions

- Does `useIdeUsage(ideType, ...)` bail out early for `windsurf`/`cursor`, or still attempts a fetch whose result is discarded? (Efficiency / noisy logs concern.)
- Is localized AM/PM (e.g. vi: "SA"/"CH") acceptable, or must output always be English "AM/PM"?
- Is there a future plan to show a "learn more / why unavailable" link beside the unsupported message? If so, plan the i18n key shape now (e.g. split into `usage_unsupported` + `usage_unsupported_reason`).
