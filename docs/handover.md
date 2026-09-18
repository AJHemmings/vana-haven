# Session Handover

## Last Updated

2026-09-18 — cloud session ending; work continues in a local session.

## Current Phase

Building out the v1 feature modules in the design spec's phasing order
(§12): Core scaffolding → scraper → Jobs/Gear → Key Item Cooldowns →
**Dailies (just finished)** → **Limbus (in progress, not started
coding yet)**.

## Design spec — not in this repo, read it first

The authoritative spec is `docs/superpowers/specs/2026-09-07-vana-haven-design.md`.
It is **deliberately gitignored** (`docs/superpowers/` and `.superpowers/` in
`.gitignore` — "AI-reference only — specs and plans must never be committed")
so it only exists on whichever machine last had it. If the new local session
doesn't have it on disk, paste its contents in early — it was pasted into
the cloud chat this session and is the single source of truth for scope,
goals, and the per-module design (§8.1–§8.6), including two since-resolved
revisions baked into it: Dailies became a free-form todo list (§8.4, no
auto-detection) and the Currency Dashboard was dropped entirely (§8.5, use
Alexandria instead).

There's also a KI-cooldowns-specific design doc referenced from the main
spec: `docs/superpowers/specs/2026-09-15-key-item-cooldowns-design.md` —
same story, gitignored, local-only, already implemented (see below).

## What Was Just Worked On

**Dailies module (§8.4)** — done, tested, pushed to
`claude/affectionate-bohr-81upse`:

- `src-tauri/src/db.rs`: `daily_todo_items` table (per-character,
  `text` + `cadence` + `last_completed_at`) and a generic `app_settings`
  key/value table holding `monthly_cycle_started_at`.
- `src-tauri/src/lib.rs`: 6 new Tauri commands (`get_daily_todo_items`,
  `create_daily_todo_item`, `delete_daily_todo_item`,
  `set_daily_todo_completed`, `get_monthly_cycle_started_at`,
  `advance_monthly_cycle`).
- `src/dailies.ts`: pure boundary math — daily reset = 15:00 UTC (JST
  midnight), weekly reset = Sunday JST midnight (Saturday 15:00 UTC) —
  computed client-side from raw timestamps, same split of responsibility
  as the Key Items countdown (backend stores facts, frontend derives
  "is this done right now").
- Monthly cadence has **no formula boundary** — a version update's date
  isn't computable, so it's a manually-advanced marker
  (`monthly_cycle_started_at`) the user bumps via a "version update
  happened" button, only shown when a monthly item exists.
- `src/DailiesScreen.tsx` + routing (`App.tsx`) + hub link
  (`CharacterHub.tsx`).
- Tests: 8 new Rust tests (79 total passing), 24 new frontend tests (62
  total passing). `npx tsc --noEmit` clean.

Commits on `claude/affectionate-bohr-81upse` (already pushed):
- `c33988c` — `fix: add missing icons/icon.png so the app builds on Linux`
- `3dac5cf` — `feat: add Dailies module (per-character free-form todo list)`

Both are on top of `3edd87c` (the already-merged Key Item Cooldowns PR on
`main`). No PR has been opened for this branch yet — not asked for.

## Decisions Made This Session

- **Dailies item lifecycle**: items are recurring templates (added once,
  stay forever); only the done/not-done flag resets each period. Confirmed
  by the user over an ephemeral-per-period alternative.
- **Dailies scope**: per-character, matching every other module (Jobs,
  Gear, Key Items) — not shared across the roster.
- **Cadence**: daily/weekly/monthly per item (not just daily as the spec
  text literally says) — user wants all three.
- **Monthly reset mechanism**: manual marker, not a formula — there's no
  fixed calendar rule for version-update dates, so auto-detection is a
  dead end here (consistent with the app's "manual fallback only where no
  clean signal exists" goal).
- **Missing `icons/icon.png` was a real, pre-existing bug**, unrelated to
  Dailies: `tauri::generate_context!()` needs it to exist at compile time
  on any platform, and only `icon.ico` had ever been committed, so a
  fresh Linux clone couldn't `cargo build`/`test` at all. Fixed as its own
  commit (derived from the existing `icon.ico`, not a new asset).
- **No AI-assistant attribution anywhere** — explicit user instruction.
  Commit messages, and the git author/committer identity itself, were
  rewritten (soft-reset + recommit with a normal author identity +
  force-push) after this surfaced partway through the session. Apply
  this to every future commit and any PR body: no AI-assistant mentions,
  and check `git log --format='%an <%ae>'` after committing, not just the
  message text — author/committer identity is separate metadata that a
  message-only check won't catch.

## Unfinished Work — Limbus module (§8.6)

**No code written yet.** The spec's original §8.6/§5.2 plan (porting
LimbusTracker's full wing/chest/currency tracking) was explicitly
**rejected by the user** — they don't want LimbusTracker's feature set
carried over, just a much smaller surface. Clarifying questions were
asked and answered this session; here's the resolved scope:

- **Two zones tracked independently**: Temenos and Apollyon.
- **What the app needs to know per zone**: (1) whether the current
  run/climb has had all floors cleared, and (2) how many times it's been
  cleared **this week** (a count against the weekly cap, e.g. "3/5" —
  not a lifetime total).
- **Weekly cap is per zone, not shared**: 5 entries per zone per week — 4
  "climbs" (normal entry) + 1 "code" (a separate entry type). So 5+5=10
  total across both zones, not 5 combined. Confirmed explicitly after an
  earlier wrong guess (don't re-guess FFXI mechanics — ask).
- **Climb vs. code is addon-detectable**: there is a specific item that's
  collected and then consumed/used to make a "code" entry — so which
  entry type happened should be inferable from that item's
  presence/consumption, not a manual toggle. Exact item name/ID: **not
  yet identified** — next step below.
- **"All floors cleared" detection signal**: the user pointed at the
  LimbusTracker Windower addon itself as the place to find this — i.e.
  don't guess from BG-Wiki or general knowledge, read the actual addon
  source. That addon lives on the user's **local machine**, not in this
  cloud container (earlier research in `docs/npc-storage-research.md`
  referenced it at a path like `E:\ffxi\addons\limbustracker\...` on a
  previous local session) — which is exactly why this module needs a
  local session to continue: the cloud container has no access to it.

### Immediate Next Steps

1. On the local machine, open `E:\ffxi\addons\limbustracker\` (or
   wherever it actually sits — confirm the path) and read its source for:
   - The key item ID(s)/table that signal "all floors cleared" for a
     Temenos run and for an Apollyon run (LimbusTracker's wing ID tables
     are the known starting point, per the earlier NPC-storage research
     doc's citation of `Alexandria.lua`-style patterns — but this time
     confirm directly from LimbusTracker's own source, not by inference).
   - The specific item that's collected/consumed for a "code" entry (name
     + item ID), so the addon can detect climb-vs-code automatically.
2. With those IDs in hand, design the schema: almost certainly a
   `character_limbus_progress`-style table per character per zone,
   holding this week's climb count, code count (capped 4 and 1), and
   whatever "last run fully cleared" state needs to derive the
   all-floors-cleared signal from held/lost key items — reuse the
   existing key-item-transition-detection pattern in `db.rs`
   (`report_key_items_held`) rather than inventing a new one if it fits.
3. Weekly reset: reuse `weeklyBoundary()` from `src/dailies.ts` (Sunday
   JST midnight) rather than re-deriving it — same boundary rule applies
   here.
4. Write the addon-side Lua changes, protocol message, backend
   commands/tests, frontend screen/tests — same shape as every prior
   module (Jobs → Gear → Key Items → Dailies).
5. Update `docs/handover.md` (this file) again at the end of that
   session, prepending above this entry — don't delete this history.

## Errors Hit and Resolutions

- **`cargo test` failed**: `gdk-sys` build script couldn't find
  `gdk-3.0.pc` via pkg-config. Fixed by installing GTK/WebKit dev
  packages (`libgtk-3-dev libwebkit2gtk-4.1-dev librsvg2-dev
  libayatana-appindicator3-dev libsoup-3.0-dev`) via apt — a first-run
  cloud-container gap, not a code issue. Likely irrelevant on a local
  Windows machine (different Tauri backend), but worth knowing if
  building on Linux again.
- **`cargo test`/`build` failed again after that**: `tauri::generate_context!()`
  panicked because `icons/icon.png` didn't exist. Root-caused as a
  genuine pre-existing repo gap (see above) rather than an
  environment-only issue, and fixed with a real commit.
- **`npx vitest` failed initially**: `node_modules` wasn't installed yet
  in the fresh container. Fixed with `npm install`. Watch out for
  `package-lock.json` picking up cosmetic `"peer": true` churn from a
  different local npm version — don't commit that noise if it reappears
  (`git checkout -- package-lock.json` before committing).

## Open Questions

- Exact LimbusTracker addon path and file(s) to read on the local
  machine (assumed `E:\ffxi\addons\limbustracker\` based on the earlier
  NPC-storage research doc's citation style — confirm, don't assume).
- Once the floor-clear and code-item IDs are known: does "all floors
  cleared" mean every wing in the zone, or just reaching the final
  floor/boss of whichever wing was entered? Confirm from the addon
  source rather than guessing.

---
