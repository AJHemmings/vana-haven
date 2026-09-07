# NPC-Stored Key Item Count — Research Finding

**Question:** Can Vana Haven read NPC-stored key item counts (e.g. the Omen
canteen: 3 storable on the NPC + 1 on-character = 4 total) automatically via
Windower, or only when the player actively opens that NPC's menu?

## Method

No live FFXI client/Windower session was available to capture packets directly
(the "go stand at Incantrix and run `//packet`" step in the task). Instead,
three genuinely available local sources were cross-referenced:

1. **Windower's own packet field definitions** —
   `E:\ffxi\addons\libs\packets\fields.lua` (the actual Windower 4 `packets`
   resource library installed on this machine, the same library `windower.ffxi.*`
   addons — including our own planned addon — depend on for packet parsing).
2. **Alexandria's working addon source** —
   `F:\Projects\Alexandria\addon\Alexandria\Alexandria.lua` — the exact
   reference implementation this project is modeled on (design spec §5.1),
   which already implements a currency-reading feature using this mechanism in
   production.
3. **BG-Wiki + community documentation**, via web search, to confirm the
   specific game mechanic (Incantrix / Mystical Canteen) and independently
   verify what the in-game "Currencies 2" menu displays.

## Finding

**The Omen canteen case is resolved: yes, fully auto-readable, no NPC visit
required.**

The item in question is the **Mystical Canteen** key item, granted by the NPC
**Incantrix** (Reisenjima). BG-Wiki confirms the exact numbers in the task
description: "Three can be stored on Incantrix, and one can be held" — 3 NPC +
1 on-character = 4 total, one regenerated every 20 real-world hours.

Windower's packet library defines an incoming packet **`0x118`**, labeled
`"Currency Info (Currencies2)"`
(`E:\ffxi\addons\libs\packets\fields.lua:3996`), which includes a field:

```lua
{ctype='unsigned char', label='Mystical Canteens'},  -- offset 0x0B
```

This packet is the response to two outgoing request packets Windower also
documents:

- `0x10F` — `"Currency Menu"` (`fields.lua:1201`)
- `0x115` — `"Currency 2 Menu"` (`fields.lua:1236`)

These correspond to the FFXI **main menu → Currencies / Currencies 2** panel —
a general account-status screen reachable from the ESC menu from anywhere in
the game world, **not** any dialogue tied to Incantrix specifically. A second
web search confirmed the in-game semantics directly: the "Currencies 2" menu's
Mystical Canteen count **is the combined total** (what you're carrying +
what's stored with Incantrix) — i.e. exactly the "4 total" figure the task
description uses as its example, already merged into a single number by the
game server itself.

Critically, **Alexandria's own shipping addon already automates this today**,
with no player interaction beyond running the addon
(`Alexandria.lua:1691-1698`):

```lua
function currency_request()
    if not packets_ok then return end
    local now = os.clock()
    if now - currency_req_t < 2 then return end
    currency_req_t = now
    pcall(function() packets.inject(packets.new('outgoing', 0x10F)) end)
    coroutine.schedule(function() if packets_ok then pcall(function()
        packets.inject(packets.new('outgoing', 0x115)) end) end
    end, 1)
end
```

It injects the two "open currency menu" request packets itself (the player
never opens any menu, and never needs to be near Incantrix or any NPC), then
reads the resulting `0x118` response and reports `"Mystical Canteens"` through
its own `CURRENCY_FIELDS` table (`Alexandria.lua:1477-1542`,
`build_currency()` at `1671-1689`) alongside Temenos/Apollyon Units — the
exact packet the design spec (§4.1) already plans to use for Limbus currency
tracking. So this isn't a theoretical read of a field definition; it's a
technique already proven working in a shipping addon built on the same
Windower API surface Vana Haven's addon will use.

**Net result:** the "3 on NPC + 1 on-character = 4 total" figure for the Omen
canteen is retrievable purely by injecting two harmless outgoing packets and
reading the response — from anywhere, at any time, with no player action and
no NPC proximity required. This fits the "full auto-detection wherever the
game/Windower exposes the data" goal (design spec §2) without any fallback.

## Caveat — this is proven for Mystical Canteen specifically, not universally

The `0x118` packet is a curated table of specific named counters (Bayld,
Kinetic Units, Mystical Canteens, several "…Stones Stored" / "…Wings Stored"
fields, etc. — `fields.lua:3996-4020+`), not a generic "any NPC storage"
mechanism. Other key items with an NPC-storage component may or may not have
an equivalent field in `0x118` or a sibling packet — that has to be checked
per key item as they're catalogued (grep `fields.lua` for the item's expected
label, the same way this research did for "Canteen"). Nothing here proves
every NPC-storage key item is auto-readable — only that the *specific example*
the spec cites, and by extension the general *class* of "account-side reserve
counter" mechanic (Ghastly/Verdigris/Wailing/Snowslit/Snowtip Stones,
Lebondopt/Pulchridopt Wings, Silver A.M.A.N. Vouchers — all "…Stored" fields
in the same packet), is a well-established, passively-readable pattern in this
game's packet protocol, not an edge case.

## Implication for `KeyItemDefinition` (spec §6/§8.3)

**No `requires_manual_refresh` flag or `last_checked_at` timestamp is needed
as a blanket schema addition** — the specific Omen canteen case the spec
flagged as the open risk is fully auto-readable using the same mechanism
already planned for Limbus currencies, via the addon proactively injecting
`0x10F`/`0x115` and reading `0x118`.

What the schema *should* carry, given the caveat above, is a small
**per-key-item read-mechanism marker** rather than a single global flag —
something like a `read_method` column on `KeyItemDefinition`
(`boolean_presence` for the plain `windower.ffxi.get_key_items()` list,
`currency_packet` for `0x118`-style numeric counters like Mystical Canteens,
or `manual_only` as the fallback for any key item where neither is confirmed
to exist). This keeps the "full auto" architecture as the default and reserves
the manual "I just checked, here's the count" UI action only for whichever
individual key items genuinely turn out to have no packet signal — to be
confirmed per-item as the KeyItemDefinition reference table is populated
(scraper tool, §4.4), the same "confirm as you go" approach the spec already
uses for per-activity Dailies detection (§11).

## Sources

- [Mystical canteen - FFXI Wiki (BG-Wiki)](https://www.bg-wiki.com/ffxi/Mystical_canteen)
- Web search: FFXI "Currencies 2" main menu panel confirming combined-total display for Mystical Canteens
- `E:\ffxi\addons\libs\packets\fields.lua` (Windower 4 `packets` resource library, local install)
- `F:\Projects\Alexandria\addon\Alexandria\Alexandria.lua` (Alexandria addon source, local checkout)
