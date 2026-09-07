package.path = package.path .. ";../?.lua"
local protocol = require("lib/protocol")
local json = require("dkjson")

local function assert_equal(actual, expected, message)
    if actual ~= expected then
        error(("%s: expected %s, got %s"):format(message, tostring(expected), tostring(actual)))
    end
end

local handshake = json.decode(protocol.build_handshake(12345, "Gozoto"))
assert_equal(handshake.type, "handshake", "handshake type")
assert_equal(handshake.game_character_id, 12345, "handshake game_character_id")
assert_equal(handshake.name, "Gozoto", "handshake name")

local heartbeat = json.decode(protocol.build_heartbeat(12345))
assert_equal(heartbeat.type, "heartbeat", "heartbeat type")
assert_equal(heartbeat.game_character_id, 12345, "heartbeat game_character_id")

assert_equal(protocol.next_backoff_seconds(nil), 1, "backoff from nil")
assert_equal(protocol.next_backoff_seconds(1), 2, "backoff from 1")
assert_equal(protocol.next_backoff_seconds(16), 30, "backoff caps below 32")
assert_equal(protocol.next_backoff_seconds(30), 30, "backoff caps at 30")

print("protocol_test.lua: all assertions passed")
