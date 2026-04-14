---
title: "Webhook API Key Authentication Support"
description: "Add API Key field to webhook config for 3rd-party service authentication"
status: pending
priority: P2
effort: 2h
branch: main
tags: [webhook, auth, api-key, config]
created: 2026-04-10
---

# Webhook API Key Authentication Support

## Goal
Add `apiKey` field to webhook config so users can authenticate with 3rd-party services that require API Key header (e.g., `X-API-Key: dsk_...`).

## Current State
- Webhook sends `Authorization: Bearer {secret}` if secret provided
- No separate API Key field exists
- 3rd-party service (Claude Tools web dashboard) provides both Webhook URL and API Key

## Approach: Rename `secret` to `apiKey`

After analysis, the simplest approach is to **rename** the existing `secret` field concept. Currently `secret` sends as `Bearer {secret}`. But most webhook consumers (including the target service) expect a simple API Key via `X-API-Key` header, not Bearer token.

**Decision:** Add `apiKey` field, send via `X-API-Key` header. Keep `secret` for Bearer token. Both optional.

## Phases

| # | Phase | Status | Effort |
|---|-------|--------|--------|
| 1 | [Backend: Add apiKey param to Rust command](phase-01-backend-rust-apikey.md) | pending | 30m |
| 2 | [Frontend: Types + Config hook + Sender hook](phase-02-frontend-types-hooks.md) | pending | 30m |
| 3 | [UI: Add API Key input to webhook settings panel](phase-03-ui-apikey-input.md) | pending | 30m |
| 4 | [i18n + Sample payload update](phase-04-i18n-sample-payload.md) | pending | 15m |
| 5 | [Compile check + Test](phase-05-compile-test.md) | pending | 15m |

## Key Decisions
- `apiKey` sent via `X-API-Key` header (industry standard for API key auth)
- `secret` kept as `Authorization: Bearer` (backward compat)
- Both fields optional, can use either or both
- UI places API Key field prominently (above secret) since it's the primary auth method for target service
