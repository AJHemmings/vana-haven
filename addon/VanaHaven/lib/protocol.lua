local protocol = {}
local json = require("dkjson")

function protocol.build_handshake(game_character_id, name)
    return json.encode({ type = "handshake", game_character_id = game_character_id, name = name }) .. "\n"
end

function protocol.build_heartbeat(game_character_id)
    return json.encode({ type = "heartbeat", game_character_id = game_character_id }) .. "\n"
end

function protocol.build_job_levels(game_character_id, main_job_id, sub_job_id, jobs)
    return json.encode({
        type = "job_levels",
        game_character_id = game_character_id,
        main_job_id = main_job_id,
        sub_job_id = sub_job_id,
        jobs = jobs,
    }) .. "\n"
end

function protocol.build_character_items(game_character_id, items)
    return json.encode({
        type = "character_items",
        game_character_id = game_character_id,
        items = items,
    }) .. "\n"
end

-- Global — no game_character_id — the whole game's key-item list, not
-- scoped to any one character.
function protocol.build_key_item_catalog(key_items)
    return json.encode({
        type = "key_item_catalog",
        key_items = key_items,
    }) .. "\n"
end

function protocol.build_key_items_held(game_character_id, key_item_ids)
    return json.encode({
        type = "key_items_held",
        game_character_id = game_character_id,
        key_item_ids = key_item_ids,
    }) .. "\n"
end

-- Exponential backoff capped at 30s: 1, 2, 4, 8, 16, 30, 30, ...
function protocol.next_backoff_seconds(previous_seconds)
    if not previous_seconds or previous_seconds <= 0 then
        return 1
    end
    return math.min(previous_seconds * 2, 30)
end

return protocol
