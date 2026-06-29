---
phase: 3
title: "Cost analytics — UI"
status: pending
priority: P2
effort: "1d"
dependencies: [2]
---

# Phase 3: Cost analytics — UI

## Overview
Dựng UI Usage dùng recharts: bar chart cost theo ngày, bảng model + session, stat tiles, range buttons, price-status badge. Style shadcn + design tokens, tham khảo bố cục cc-switch.

## Requirements
- Functional: range buttons 7d/30d/90d/All; bar chart cost theo ngày (≤45 bars); 4 stat tiles (total cost, today cost, output tokens, cache read); bảng model (sort theo tokens desc); bảng session (30 gần nhất); badge price status.
- Non-functional: dùng design tokens hiện có (dark/light); component < 200 dòng (tách nhỏ); chỉ Claude Code (bỏ Codex tab).

## Architecture
- Thêm dependency `recharts` (pnpm add recharts).
- TS types trong `src/lib/types.ts`: `TokenBreakdown`, `DayUsage`, `ModelUsage`, `SessionUsage`, `ToolUsage`, `UsageReport` (khớp serde camelCase backend).
- Hook `use-usage-report.ts`: gọi `get_usage(rangeDays)`, listen `usage-changed` refetch.
- Components (shadcn Card/Tabs/Badge/Button):
  - `usage-view.tsx` (container: range buttons + tiles + chart + tables) — giữ mỏng, compose.
  - `usage-cost-chart.tsx` (recharts BarChart, tooltip ngày/tokens/cost).
  - `usage-model-table.tsx`, `usage-session-table.tsx`.
- Entry: thêm tab "Usage" cạnh tabs Claude/IDE trong `dashboard.tsx` (xác nhận với UQ#2 — đề xuất tab mới).

## Related Code Files
- Create: `src/components/usage/usage-view.tsx`
- Create: `src/components/usage/usage-cost-chart.tsx`
- Create: `src/components/usage/usage-model-table.tsx`
- Create: `src/components/usage/usage-session-table.tsx`
- Create: `src/hooks/use-usage-report.ts`
- Modify: `src/lib/types.ts` (thêm usage types)
- Modify: `src/pages/dashboard.tsx` (thêm tab Usage)
- Modify: `package.json` (recharts)
- Read source: `scratchpad/ai-switcher/src/UsageView.tsx` (tham chiếu UI/logic)

## Implementation Steps
1. `pnpm add recharts`; thêm TS types vào types.ts.
2. Viết `use-usage-report.ts` (invoke + listen usage-changed).
3. Viết `usage-cost-chart.tsx` (recharts, downsample ≤45 ngày, tooltip).
4. Viết `usage-model-table.tsx` + `usage-session-table.tsx`.
5. Viết `usage-view.tsx` compose tiles + range buttons + chart + tables + price badge.
6. Wire tab "Usage" vào dashboard.tsx.
7. `pnpm build` / `tsc` pass; kiểm tra dark/light + responsive.

## Success Criteria
- [ ] Range buttons đổi dữ liệu (7/30/90/All)
- [ ] Chart hiển thị cost theo ngày, tooltip đúng; >45 ngày tự downsample
- [ ] Tiles + bảng model + bảng session đúng số liệu từ `UsageReport`
- [ ] Badge price status: Live / Saved / Hidden (ẩn cột cost khi Hidden)
- [ ] Theme khớp design tokens; mỗi component < 200 dòng; build pass

## Risk Assessment
- recharts bundle size → chấp nhận (1 dep), import gọn theo component cần.
- Nếu cost ẩn (Hidden) → chart fallback hiển thị token count thay vì cost (như source).
