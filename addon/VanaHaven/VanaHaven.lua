_addon.name = "VanaHaven"
_addon.author = "Adam Hemmings"
_addon.version = "0.1.0"
_addon.commands = { "vanahaven", "vh" }

local socket = require("socket")
local protocol = require("lib/protocol")

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

windower.register_event("prerender", function()
    if connecting_sock then
        finish_connect()
    elseif not sock then
        try_connect()
    else
        send_heartbeat()
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
