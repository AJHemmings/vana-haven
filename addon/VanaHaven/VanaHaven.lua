_addon.name = "VanaHaven"
_addon.author = "Adam Hemmings"
_addon.version = "0.1.0"
_addon.commands = { "vanahaven", "vh" }

local socket = require("socket")
local packets = require("packets")
local protocol = require("lib/protocol")
local res = require('resources')

local HOST = "127.0.0.1"
local DEFAULT_PORT = 24244
local PORT_FILE_SUFFIX = "\\com.vanahaven.desktop\\port.txt"

local sock = nil
local connecting_sock = nil
local connecting_since = nil
local backoff_seconds = nil
local last_attempt = 0
local last_heartbeat = 0
local HEARTBEAT_INTERVAL = 5
-- Packet 0x01B only fires on login/job-change, not on every TCP reconnect.
-- Caching the last-built job-levels payload lets us resend it once after a
-- successful (re)connect (see finish_connect) so a restarted app still gets
-- the player's current job levels without needing a fresh 0x01B to fire.
local last_job_levels_payload = nil
-- Mirrors last_job_levels_payload above: caching the last-built
-- character_items payload lets finish_connect resend the last-known
-- inventory/equipment snapshot once after a successful (re)connect, the
-- same way it already does for job levels. Declared here (not down near
-- flush_inventory_if_dirty/send_character_items, where the rest of the
-- inventory-reading code lives) because finish_connect needs to see it as
-- an upvalue, and Lua locals are only visible to code that appears after
-- their declaration in the file — declaring them near their other use
-- would silently make finish_connect read/write unrelated globals instead.
local last_character_items_payload = nil
-- Debounce window for inventory dirty-flushing (see flush_inventory_if_dirty
-- below) — declared here, not next to INV_MAX_WAIT/flush_inventory_if_dirty,
-- for the same forward-visibility reason as last_character_items_payload
-- above: finish_connect resets the debounce clock on reconnect too.
local INV_DEBOUNCE = 0.6
local inv_dirty = false
local inv_dirty_at = 0
local inv_first_dirty = nil
-- Neither a successful nor a failed non-blocking connect reliably shows up
-- via socket.select()'s write-set in this runtime (confirmed via live
-- testing) — connect completion is polled via getpeername() instead (see
-- finish_connect), so both outcomes are handled by timing out a pending
-- connect ourselves rather than waiting on select() to report anything.
local CONNECT_TIMEOUT = 3

local function player_info()
    local player = windower.ffxi.get_player()
    local info = windower.ffxi.get_info()
    if not player or not info then return nil end
    return { id = info.character_id or player.id, name = player.name }
end

-- Mirrors the fallback tools/mock-addon-client.mjs already implements: prefer
-- the port the app wrote (it only writes this file when DEFAULT_PORT was
-- taken), otherwise use the default.
local function resolve_port()
    local appdata = os.getenv("APPDATA")
    if not appdata then return DEFAULT_PORT end

    local file = io.open(appdata .. PORT_FILE_SUFFIX, "r")
    if not file then return DEFAULT_PORT end

    local content = file:read("*a")
    file:close()
    return tonumber((content:gsub("%s+", ""))) or DEFAULT_PORT
end

local function try_connect()
    if sock or connecting_sock then return end
    local now = os.clock()
    if backoff_seconds and now - last_attempt < backoff_seconds then return end
    last_attempt = now

    local port = resolve_port()
    local s = socket.tcp()
    s:settimeout(0)
    local ok, err = s:connect(HOST, port)
    if ok or err == "timeout" then
        connecting_sock = s
        connecting_since = now
    else
        s:close()
        backoff_seconds = protocol.next_backoff_seconds(backoff_seconds)
    end
end

local function fail_pending_connect()
    connecting_sock:close()
    connecting_sock = nil
    connecting_since = nil
    backoff_seconds = protocol.next_backoff_seconds(backoff_seconds)
end

local function finish_connect()
    if not connecting_sock then return end

    -- socket.select()'s write-set was observed NOT reliably signaling
    -- readiness for a connecting socket in this runtime (confirmed via
    -- live in-game testing: it never fired within CONNECT_TIMEOUT even for
    -- connections that genuinely completed at the TCP level). Polling
    -- getpeername() directly avoids depending on select() here at all — it
    -- fails until the handshake completes, then succeeds once it has.
    local peer = connecting_sock:getpeername()
    if peer then
        sock = connecting_sock
        connecting_sock = nil
        connecting_since = nil
        backoff_seconds = nil
        local info = player_info()
        local handshake_ok = true
        if info then
            handshake_ok = sock:send(protocol.build_handshake(info.id, info.name)) ~= nil
        end
        if handshake_ok then
            windower.add_to_chat(207, "[VanaHaven] connected")
            if last_job_levels_payload then
                sock:send(last_job_levels_payload)
            end
            if last_character_items_payload then
                sock:send(last_character_items_payload)
            end
            inv_dirty = true
            inv_dirty_at = os.clock() - INV_DEBOUNCE
            inv_first_dirty = os.clock() - INV_DEBOUNCE
        else
            sock:close()
            sock = nil
            backoff_seconds = protocol.next_backoff_seconds(backoff_seconds)
        end
        return
    end

    -- Not connected yet — getpeername() fails until the handshake completes
    -- (or forever, if the connect was refused). Give up after a few seconds
    -- either way rather than waiting indefinitely.
    if os.clock() - connecting_since > CONNECT_TIMEOUT then
        fail_pending_connect()
    end
end

local function send_heartbeat()
    if not sock then return end
    local now = os.clock()
    if now - last_heartbeat < HEARTBEAT_INTERVAL then return end
    last_heartbeat = now

    local info = player_info()
    if not info then return end
    local ok = sock:send(protocol.build_heartbeat(info.id))
    if not ok then
        sock:close()
        sock = nil
    end
end

-- Job id -> full FFXI job name, as used by the keys packets.parse('incoming', data)
-- produces for packet 0x01B (see docs/job-levels-packet-research.md). Index = job id,
-- matching the canonical order src/jobs.ts's JOBS array already uses.
local JOB_NAMES = {
    [1] = "Warrior",
    [2] = "Monk",
    [3] = "White Mage",
    [4] = "Black Mage",
    [5] = "Red Mage",
    [6] = "Thief",
    [7] = "Paladin",
    [8] = "Dark Knight",
    [9] = "Beastmaster",
    [10] = "Bard",
    [11] = "Ranger",
    [12] = "Samurai",
    [13] = "Ninja",
    [14] = "Dragoon",
    [15] = "Summoner",
    [16] = "Blue Mage",
    [17] = "Corsair",
    [18] = "Puppetmaster",
    [19] = "Dancer",
    [20] = "Scholar",
    [21] = "Geomancer",
    [22] = "Rune Fencer",
}

-- p is the table returned by packets.parse('incoming', data) for the 0x01B chunk
-- that triggered this. It's passed in explicitly rather than read from an outer
-- closure since it only exists for the duration of that one incoming-chunk event.
--
-- Main/sub job ids come from p["Main Job"]/p["Sub Job"] (the same packet), NOT
-- from windower.ffxi.get_player() — confirmed live (Task 14 Step 3) that the
-- player object still reports the PREVIOUS job for one event after a job
-- change, while the packet's own fields already carry the new one. Reading
-- from get_player() here reproduced an off-by-one bug: the UI always lagged
-- one job change behind.
local function send_job_levels(p)
    local info = player_info()
    if not info then return end

    local jobs = {}
    for job_id = 1, 22 do
        local name = JOB_NAMES[job_id]
        table.insert(jobs, {
            job_id = job_id,
            level = p[name .. " Level"] or 0,
            master_level = p[name .. " Master Level"] or 0,
            mastered = p[name .. " Master"] or false,
        })
    end

    local payload = protocol.build_job_levels(info.id, p["Main Job"] or 0, p["Sub Job"] or 0, jobs)
    last_job_levels_payload = payload
    if sock then
        sock:send(payload)
    end
end

-- container id -1 mirrors db::EQUIPPED_CONTAINER on the app side — a
-- distinct sentinel, not one of Windower's real bag ids (all 0-16).
local EQUIPPED_CONTAINER = -1

-- Packet ids that mark inventory/bag contents as needing a re-scan.
-- Ported from Alexandria's addon (F:\Projects\Alexandria\addon\Alexandria\
-- Alexandria.lua), an existing, working FFXI/Windower companion addon that
-- already solved this — not independently derived or guessed.
local INVENTORY_DIRTY_PACKET_IDS = { [0x01D] = true, [0x01E] = true, [0x01F] = true, [0x020] = true }
local INV_MAX_WAIT = 1.5
local inv_last_sent_key = nil

-- Reads every bag Windower's own resource table knows about (not a
-- hand-picked list) and flags each held item with its container: the
-- owning bag id, or EQUIPPED_CONTAINER if Windower reports it as
-- currently worn. Ported from Alexandria's build_inventory(), trimmed to
-- only what vana-haven needs (item_id + container, not name/count/
-- augments/etc — Alexandria's own protocol carries much more per-item
-- detail this app has no use for).
local function collect_held_items()
    local items = {}
    for bag_id, bag in pairs(res.bags) do
        local bag_items = windower.ffxi.get_items(bag_id)
        -- Mog House storage bags (Safe=1, Storage=2, Temporary=3, Locker=4,
        -- Safe 2=9) report enabled=false whenever the player isn't standing
        -- at a Mog House/Nomad Moogle, even though their cached contents
        -- are still readable — gating on `enabled` for these hides them for
        -- anyone out in the field. Alexandria's own changelog calls this
        -- "the 0.0.17 bug" before it was fixed this way; porting the fix,
        -- not just the bug's absence.
        local mog_bag = bag_id == 1 or bag_id == 2 or bag_id == 3 or bag_id == 4 or bag_id == 9
        -- Alexandria's build_inventory() also unconditionally scans bag id 17
        -- regardless of `enabled` (alongside the 5 Mog House bags above) — that
        -- exception was deliberately NOT ported here. What bag 17 actually
        -- represents in Windower's resource table is unverified (it's absent
        -- from Alexandria's own bagNames.ts display list too, same as bag 3);
        -- if a character's items are ever missing from a report while out in
        -- the field, this is the first place to check live, not guess at here.
        if type(bag_items) == 'table' and (bag_items.enabled or (mog_bag and (bag_items.count or 0) > 0)) then
            local maxn = bag_items.max or 0
            for s = 1, maxn do
                local it = bag_items[s]
                if it and it.id and it.id ~= 0 then
                    -- status 5 = currently equipped (confirmed against
                    -- Alexandria, which uses this same status value to
                    -- detect locked/equipped slots it refuses to move).
                    -- Equipped items live in the SAME bag slot arrays as
                    -- everything else — Windower has no separate
                    -- "equipment" read path, contrary to this app's
                    -- original design-spec assumption (§4).
                    local container = (it.status == 5) and EQUIPPED_CONTAINER or bag_id
                    table.insert(items, { item_id = it.id, container = container })
                end
            end
        end
    end
    return items
end

local function inventory_key(items)
    local parts = {}
    for _, it in ipairs(items) do
        table.insert(parts, it.item_id .. ':' .. it.container)
    end
    table.sort(parts)
    return table.concat(parts, ',')
end

-- Mirrors send_job_levels's cache-then-send-if-connected pattern: the
-- payload is built and deduped regardless of connection state, so a
-- reconnect can resend the last-known snapshot (see finish_connect).
local function send_character_items()
    local info = player_info()
    if not info then return end
    local items = collect_held_items()
    local key = inventory_key(items)
    if key == inv_last_sent_key then return end
    inv_last_sent_key = key
    local payload = protocol.build_character_items(info.id, items)
    last_character_items_payload = payload
    if sock then sock:send(payload) end
end

local function flush_inventory_if_dirty()
    if not inv_dirty then return end
    local now = os.clock()
    if now - inv_dirty_at < INV_DEBOUNCE and now - inv_first_dirty < INV_MAX_WAIT then return end
    inv_dirty = false
    inv_first_dirty = nil
    send_character_items()
end

windower.register_event("incoming chunk", function(id, data)
    if id == 0x01B then
        local p = packets.parse('incoming', data)
        if p then send_job_levels(p) end
    elseif INVENTORY_DIRTY_PACKET_IDS[id] then
        if not inv_dirty then inv_first_dirty = os.clock() end
        inv_dirty = true
        inv_dirty_at = os.clock()
    end
end)

windower.register_event("prerender", function()
    if connecting_sock then
        finish_connect()
    elseif not sock then
        try_connect()
    else
        send_heartbeat()
        flush_inventory_if_dirty()
    end
end)

windower.register_event("login", function()
    -- Re-send a fresh handshake in case the character changed while the TCP
    -- connection stayed open (e.g. switching characters via /send).
    if not sock then return end
    local info = player_info()
    if info and sock:send(protocol.build_handshake(info.id, info.name)) == nil then
        sock:close()
        sock = nil
    end
end)

windower.register_event("unload", function()
    if sock then sock:close() end
    if connecting_sock then connecting_sock:close() end
end)
