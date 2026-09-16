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

local key_item_catalog = protocol.build_key_item_catalog({
    { key_item_id = 1, name = "Rubber Cockatrice" },
    { key_item_id = 42, name = "Mystical Canteen" },
})
local decoded_kic = json.decode(key_item_catalog)
assert_equal(decoded_kic.type, "key_item_catalog", "key_item_catalog type")
assert_equal(#decoded_kic.key_items, 2, "key_item_catalog key_items count")
assert_equal(decoded_kic.key_items[1].key_item_id, 1, "key_item_catalog key_items[1].key_item_id")
assert_equal(decoded_kic.key_items[2].name, "Mystical Canteen", "key_item_catalog key_items[2].name")

local key_item_catalog_empty = protocol.build_key_item_catalog({})
local decoded_kic_empty = json.decode(key_item_catalog_empty)
assert_equal(#decoded_kic_empty.key_items, 0, "key_item_catalog empty key_items")

local key_items_held = protocol.build_key_items_held(12345, { 1, 42 })
local decoded_kih = json.decode(key_items_held)
assert_equal(decoded_kih.type, "key_items_held", "key_items_held type")
assert_equal(decoded_kih.game_character_id, 12345, "key_items_held game_character_id")
assert_equal(#decoded_kih.key_item_ids, 2, "key_items_held key_item_ids count")
assert_equal(decoded_kih.key_item_ids[2], 42, "key_items_held key_item_ids[2]")

local key_items_held_empty = protocol.build_key_items_held(12345, {})
local decoded_kih_empty = json.decode(key_items_held_empty)
assert_equal(#decoded_kih_empty.key_item_ids, 0, "key_items_held empty key_item_ids")

print("protocol_test.lua: all assertions passed")
