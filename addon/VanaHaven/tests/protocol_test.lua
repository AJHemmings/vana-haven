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

local job_levels = protocol.build_job_levels(12345, 4, 20, {
    { job_id = 1, level = 75, master_level = 0, mastered = false },
    { job_id = 22, level = 99, master_level = 12, mastered = true },
})
local decoded_jl = json.decode(job_levels)
assert_equal(decoded_jl.type, "job_levels", "job_levels type")
assert_equal(decoded_jl.game_character_id, 12345, "job_levels game_character_id")
assert_equal(decoded_jl.main_job_id, 4, "job_levels main_job_id")
assert_equal(decoded_jl.sub_job_id, 20, "job_levels sub_job_id")
assert_equal(#decoded_jl.jobs, 2, "job_levels jobs count")
assert_equal(decoded_jl.jobs[1].job_id, 1, "job_levels jobs[1].job_id")
assert_equal(decoded_jl.jobs[1].level, 75, "job_levels jobs[1].level")
assert_equal(decoded_jl.jobs[2].master_level, 12, "job_levels jobs[2].master_level")
assert_equal(decoded_jl.jobs[2].mastered, true, "job_levels jobs[2].mastered")

local job_levels_no_sub = protocol.build_job_levels(12345, 4, 0, {})
local decoded_no_sub = json.decode(job_levels_no_sub)
assert_equal(decoded_no_sub.sub_job_id, 0, "job_levels sub_job_id zero")
assert_equal(#decoded_no_sub.jobs, 0, "job_levels empty jobs")

print("protocol_test.lua: all assertions passed")
