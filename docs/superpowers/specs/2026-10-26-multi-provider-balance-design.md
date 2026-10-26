# Multi-Provider Balance Support — Design & Research Notes

**Status:** Deferred (not scheduled for implementation)
**Date:** 2026-10-26
**Scope:** Displaying balances/credits for additional LLM providers (Z.ai, PPQ.ai) alongside the existing OpenCode Go quota applet.

## Intent and why

The applet currently shows a single provider: OpenCode Go quota (rolling/weekly/monthly
percentages). The user also pays as-you-go with other providers and would like their
remaining balance visible the same way — without opening a web console. This document
records what was verified about each provider's API so future work can start from
facts, not guesses.

## Verified API findings (2026-10-26)

### PPQ.ai — usable, documented balance endpoint

- API is OpenAI-compatible, base URL `https://api.ppq.ai`, keys are `sk-…` (Bearer).
- **Documented:** `POST https://api.ppq.ai/credits/balance`
  - Auth accepted via `Authorization: Bearer <API key>`, an `api-key:` header, **or**
    `{"credit_id": "…"}` in the body (verified: all three are named by the endpoint's
    error response with an empty body).
  - Response shape: `{"balance": <USDT/USD amount>}` (verified with a fake
    `credit_id` → `{"balance":0}`; exact 401/402 error shapes documented in their
    llms.txt).
- Auth model: pay-per-query; balance enforcement returns `402 Payment Required`
  ("this request requires a minimum balance of $X … Please top up.").
- Source of truth: `https://ppq.ai/llms.txt` → section "Billing & Topups".
- **Local state:** no PPQ key exists in `~/.local/share/opencode/auth.json`; a key
  must be provided (location TBD, see Open questions).

### Z.ai — blocked, no balance API for API keys

- Official OpenAPI spec (`docs.z.ai/openapi.json`) and `llms.txt` list **no**
  billing/balance endpoint. Billing lives only in the web console
  (`z.ai/manage-apikey/billing`, "Payment Method").
- ~16 candidate paths under `https://api.z.ai/api/biz/*` probed with a real API key
  (`/balance`, `/credit/balance`, `/account/balance`, `/wallet`, `/usage`,
  `/sub/usage`, `/pay/balance`, `/user/balance`, `/billing/info`,
  `/subscription/usage`, `/order/list`, `/account/info` …): all respond
  `404 {"msg":"接口不存在…"}` from the internal API router; `/credit/balance`
  returns a CDN 405 HTML page (not an API route). No path accepts the API key.
- The web console fetches billing with **session cookies**, not the API key.
- Conclusion (same policy as OpenCode Zen): balance is only obtainable by browser
  scraping with a session cookie. That is deliberately **not implemented** — fragile
  and violates the project's no-scraping principle. Revisit only if Z.ai ships an
  official endpoint (watch their openapi.json / release notes).

## Non-goals

- No cookie/session-based scraping for any provider (incl. Z.ai and OpenCode Zen).
- No spending analytics or history charts; balance display only.

## Options considered and how the existing code would be touched

The current code is single-provider by construction: `api::fetch_quota` →
`quota::parse_status` → `QuotaSnapshot` (rolling/weekly/monthly), one cache file,
one panel line (`16% · 77% W · 38% M`), color logic driven by
`usage_level(percent_used)`. PPQ data is **absolute USD remaining**, not a percent —
color thresholds would have to be independent (e.g. yellow < $5, red < $1).

**Option A — generic `QuotaSource` abstraction (recommended).**
Introduce a small trait/enum `QuotaSource`; each provider gets its own fetcher,
key-resolution (generalized `resolve_api_key`), cache file, and display block.
OpenCode Go remains the percent-provider; PPQ becomes a balance-provider with its
own value formatting (`$4.17`) and thresholds. Scales to future providers with
documented balance endpoints (e.g. OpenRouter `/api/v1/credits`).
Trade-off: more refactoring up front (~touches api, cache, config, applet).

**Option B — hard-wire PPQ next to OpenCode Go.**
Second fetcher + second block without abstraction. Fastest path, but the next
provider duplicates the pattern again.

**Panel/popup display (decided when implementing):**
either a second segment in the same panel line (`… M · $4.17` — line is already the
longest applet in the wing) or PPQ as a popup-only row with a small/abbreviated
panel segment. History context: priority on text readability in the panel led to the
native applet in the first place, so truncation should be avoided.

## Open questions (need answers before implementation)

1. **PPQ key location:** where does the user store the PPQ API key? Candidates:
   own entry in `~/.local/share/opencode/auth.json` (reusing generalized
   `resolve_api_key`), env var (`PPQ_API_KEY`), or a dedicated config file.
2. **Panel versus popup display** (see above).
3. **Balance color thresholds** — yellow/red dollar values acceptable to the user.

## Recommended next step when resumed

Implement Option A incrementally: (1) key/config generalization, (2) PPQ fetcher +
parser behind `QuotaSource`, (3) popup row, (4) panel segment decision. With TDD per
the repo's testing conventions (`cargo test`, clippy `-D warnings`).
