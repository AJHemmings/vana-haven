# Job Info Packet (0x01B) Field Names — Research Finding

**Question:** What key name(s) does `packets.parse("incoming", data)` produce for the
job-level, job-master, and job-master-level array fields in packet `0x01B` (offsets
`0x49`, `0x68`, `0x6D` in `fields.lua`, none of which have an explicit `label=`)?

**Finding:** They are not an array or a sub-table at all. `packets.parse` flattens
all 22 jobs into 66 separate top-level scalar fields on the returned table `p`, one
triple per job, keyed by the job's full English name (not its abbreviation, not a
numeric id):

- `p["{JobName} Level"]` — integer, the job's level (0-99)
- `p["{JobName} Master"]` — boolean, whether the job is mastered
- `p["{JobName} Master Level"]` — integer, the job's Master Level (0-50)

Confirmed live in-game (via a temporary `pairs(p)` dump on an incoming `0x01B` chunk)
for every one of the 22 jobs, e.g.:

```
p["Warrior Level"] = 99            p["Warrior Master"] = true      p["Warrior Master Level"] = 48
p["Dragoon Level"] = 99            p["Dragoon Master"] = true      p["Dragoon Master Level"] = 37
p["Summoner Level"] = 1            p["Summoner Master"] = false    p["Summoner Master Level"] = 0
p["Blue Mage Level"] = 99          p["Blue Mage Master"] = false   p["Blue Mage Master Level"] = 0
p["Puppetmaster Level"] = 99       p["Puppetmaster Master"] = true p["Puppetmaster Master Level"] = 2
p["Scholar Level"] = 1             p["Scholar Master"] = false     p["Scholar Master Level"] = 0
p["Geomancer Level"] = 99          p["Geomancer Master"] = true    p["Geomancer Master Level"] = 8
```

`{JobName}` is FFXI's full in-game job name, exactly as Windower's `res/jobs` table
spells it — including the two-word names (`"White Mage"`, `"Black Mage"`, `"Red Mage"`,
`"Blue Mage"`, `"Dark Knight"`, `"Rune Fencer"`) and the no-space compound names
(`"Beastmaster"`, `"Puppetmaster"`). The full set of 22 names, in the canonical id
order already established in `src/jobs.ts`:

```
1  WAR  "Warrior"        9  BST  "Beastmaster"     17 COR  "Corsair"
2  MNK  "Monk"           10 BRD  "Bard"             18 PUP  "Puppetmaster"
3  WHM  "White Mage"     11 RNG  "Ranger"           19 DNC  "Dancer"
4  BLM  "Black Mage"     12 SAM  "Samurai"          20 SCH  "Scholar"
5  RDM  "Red Mage"       13 NIN  "Ninja"            21 GEO  "Geomancer"
6  THF  "Thief"          14 DRG  "Dragoon"          22 RUN  "Rune Fencer"
7  PLD  "Paladin"        15 SMN  "Summoner"
8  DRK  "Dark Knight"    16 BLU  "Blue Mage"
```

Also present in the same packet, as an aside (not needed for Task 12, since
`send_job_levels` already reads main/sub job id from `windower.ffxi.get_player()`):
`p["Main Job"]` and `p["Sub Job"]` — numeric job ids for the character's current
main/sub job. Everything else in the dump (`p["Base STR"]`, `p["Maximum HP"]`,
`p["Mentor Icon"]`, `p["_id"]`, `p["_name"]`, `p["_raw"]`, etc.) is unrelated
character-stat or Windower packet-metadata noise, safe to ignore.

**Implication for Task 12:** `send_job_levels()` cannot iterate `job_id = 1, 22` and
index into `p` numerically — there is no numeric or abbreviation key. It needs a
lookup table mapping each of the 22 canonical job ids to its full FFXI name (the
table above), then for each `job_id` build the entry as:

```lua
local level = p[name .. " Level"] or 0
local master_level = p[name .. " Master Level"] or 0
local mastered = p[name .. " Master"] or false
```

where `name` comes from the id→name lookup table. This lookup table needs to be
added to the addon (either inline in `VanaHaven.lua` or as a small new module) —
it doesn't exist yet anywhere in the codebase; `src/jobs.ts`'s `JOBS` array has
ids→abbreviations, not ids→full names, so it can't be reused as-is on the Lua side.
