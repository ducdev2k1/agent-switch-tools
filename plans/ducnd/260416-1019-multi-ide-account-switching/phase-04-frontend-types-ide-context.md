# Phase 4: Frontend — Types + IDE Context

**Priority:** High
**Status:** Pending
**Depends on:** Phase 3

---

## Overview

Them TypeScript types cho IDE profiles va tao hook `use-ide-profiles` de manage state cho Cursor/Antigravity accounts.

## Related Code Files

**Modify:**
- `src/lib/types.ts` — add IDE-related types

**Create:**
- `src/hooks/use-ide-profiles.ts` — hook goi Tauri commands cho IDE profiles
- `src/hooks/use-installed-ides.ts` — hook detect installed IDEs

**Read for context:**
- `src/hooks/use-profiles.ts` — existing Claude Code profile hook (pattern to follow)
- `src/lib/types.ts` — existing types

## Implementation Steps

1. Add types to `src/lib/types.ts`:

   ```typescript
   export type IdeType = 'cursor' | 'antigravity'

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
     ideType: string
   }

   export interface IdeSwitchResult {
     success: boolean
     ideWasRunning: boolean
     message: string
   }
   ```

2. Create `use-installed-ides.ts`:
   - Call `list_installed_ides` on mount
   - Cache result (IDEs don't change during session)
   - Export `installedIdes: IdeInfo[]`, `loading: boolean`

3. Create `use-ide-profiles.ts`:
   - Takes `ideType: IdeType` as parameter
   - Pattern identical to `use-profiles.ts` but calls IDE-specific commands:
     - `list_ide_profiles(ideType)` -> profiles
     - `save_current_ide_profile(ideType)` -> email
     - `switch_ide_profile(ideType, targetName)` -> result
     - `rename_ide_profile(ideType, oldName, newName)`
     - `delete_ide_profile(ideType, name)`
     - `is_ide_running(ideType)` -> boolean

## Todo

- [ ] Add IDE types to types.ts
- [ ] Create `use-installed-ides.ts` hook
- [ ] Create `use-ide-profiles.ts` hook
- [ ] Verify TypeScript compiles

## Success Criteria

- All new types compile without errors
- Hooks callable from components
- `useInstalledIdes()` returns detected IDEs
- `useIdeProfiles('cursor')` returns profile list
