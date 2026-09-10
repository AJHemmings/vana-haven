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

-- Exponential backoff capped at 30s: 1, 2, 4, 8, 16, 30, 30, ...
function protocol.next_backoff_seconds(previous_seconds)
    if not previous_seconds or previous_seconds <= 0 then
        return 1
    end
    return math.min(previous_seconds * 2, 30)
end

return protocol
