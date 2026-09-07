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
local backoff_seconds = nil
local last_attempt = 0
local last_heartbeat = 0
local HEARTBEAT_INTERVAL = 5

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

    local s = socket.tcp()
    s:settimeout(0)
    local ok, err = s:connect(HOST, resolve_port())
    if ok or err == "timeout" then
        connecting_sock = s
    else
        s:close()
        backoff_seconds = protocol.next_backoff_seconds(backoff_seconds)
    end
end

local function finish_connect()
    if not connecting_sock then return end
    local ready = socket.select(nil, { connecting_sock }, 0)
    if ready and #ready > 0 then
        sock = connecting_sock
        connecting_sock = nil
        backoff_seconds = nil
        local info = player_info()
        if info then
            sock:send(protocol.build_handshake(info.id, info.name))
        end
        windower.add_to_chat(207, "[VanaHaven] connected")
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
    if info then
        sock:send(protocol.build_handshake(info.id, info.name))
    end
end)

windower.register_event("unload", function()
    if sock then sock:close() end
    if connecting_sock then connecting_sock:close() end
end)
