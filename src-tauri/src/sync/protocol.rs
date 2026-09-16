use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub struct JobEntry {
    pub job_id: i64,
    pub level: i64,
    pub master_level: i64,
    pub mastered: bool,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub struct ItemEntry {
    pub item_id: i64,
    pub container: i64,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub struct KeyItemEntry {
    pub key_item_id: i64,
    pub name: String,
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum AddonMessage {
    Handshake { game_character_id: i64, name: String },
    Heartbeat { game_character_id: i64 },
    JobLevels {
        game_character_id: i64,
        main_job_id: i64,
        sub_job_id: i64,
        jobs: Vec<JobEntry>,
    },
    CharacterItems {
        game_character_id: i64,
        items: Vec<ItemEntry>,
    },
    // Global — no game_character_id — the whole game's key-item list, not
    // scoped to any one character. See Key Item Cooldowns spec §4/§6.
    KeyItemCatalog {
        key_items: Vec<KeyItemEntry>,
    },
    KeyItemsHeld {
        game_character_id: i64,
        key_item_ids: Vec<i64>,
    },
}

pub fn parse_message(raw: &str) -> Result<AddonMessage, serde_json::Error> {
    serde_json::from_str(raw)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_handshake_message() {
        let raw = r#"{"type":"handshake","game_character_id":12345,"name":"Gozoto"}"#;
        assert_eq!(
            parse_message(raw).unwrap(),
            AddonMessage::Handshake { game_character_id: 12345, name: "Gozoto".to_string() }
        );
    }

    #[test]
    fn parses_heartbeat_message() {
        let raw = r#"{"type":"heartbeat","game_character_id":12345}"#;
        assert_eq!(
            parse_message(raw).unwrap(),
            AddonMessage::Heartbeat { game_character_id: 12345 }
        );
    }

    #[test]
    fn rejects_malformed_json() {
        assert!(parse_message("not json").is_err());
    }

    #[test]
    fn parses_job_levels_message() {
        let raw = r#"{"type":"job_levels","game_character_id":12345,"main_job_id":4,"sub_job_id":20,"jobs":[{"job_id":1,"level":75,"master_level":0,"mastered":false},{"job_id":22,"level":99,"master_level":12,"mastered":true}]}"#;
        assert_eq!(
            parse_message(raw).unwrap(),
            AddonMessage::JobLevels {
                game_character_id: 12345,
                main_job_id: 4,
                sub_job_id: 20,
                jobs: vec![
                    JobEntry { job_id: 1, level: 75, master_level: 0, mastered: false },
                    JobEntry { job_id: 22, level: 99, master_level: 12, mastered: true },
                ],
            }
        );
    }

    #[test]
    fn parses_job_levels_message_with_zero_sub_job_id() {
        let raw = r#"{"type":"job_levels","game_character_id":12345,"main_job_id":4,"sub_job_id":0,"jobs":[]}"#;
        assert_eq!(
            parse_message(raw).unwrap(),
            AddonMessage::JobLevels {
                game_character_id: 12345,
                main_job_id: 4,
                sub_job_id: 0,
                jobs: vec![],
            }
        );
    }

    #[test]
    fn parses_character_items_message() {
        let raw = r#"{"type":"character_items","game_character_id":12345,"items":[{"item_id":15079,"container":0},{"item_id":27683,"container":-1}]}"#;
        assert_eq!(
            parse_message(raw).unwrap(),
            AddonMessage::CharacterItems {
                game_character_id: 12345,
                items: vec![
                    ItemEntry { item_id: 15079, container: 0 },
                    ItemEntry { item_id: 27683, container: -1 },
                ],
            }
        );
    }

    #[test]
    fn parses_character_items_message_with_empty_items() {
        let raw = r#"{"type":"character_items","game_character_id":12345,"items":[]}"#;
        assert_eq!(
            parse_message(raw).unwrap(),
            AddonMessage::CharacterItems { game_character_id: 12345, items: vec![] }
        );
    }

    #[test]
    fn parses_key_item_catalog_message() {
        let raw = r#"{"type":"key_item_catalog","key_items":[{"key_item_id":1,"name":"Rubber Cockatrice"},{"key_item_id":2,"name":"Mystical Canteen"}]}"#;
        assert_eq!(
            parse_message(raw).unwrap(),
            AddonMessage::KeyItemCatalog {
                key_items: vec![
                    KeyItemEntry { key_item_id: 1, name: "Rubber Cockatrice".to_string() },
                    KeyItemEntry { key_item_id: 2, name: "Mystical Canteen".to_string() },
                ],
            }
        );
    }

    #[test]
    fn parses_key_item_catalog_message_with_empty_key_items() {
        let raw = r#"{"type":"key_item_catalog","key_items":[]}"#;
        assert_eq!(parse_message(raw).unwrap(), AddonMessage::KeyItemCatalog { key_items: vec![] });
    }

    #[test]
    fn parses_key_items_held_message() {
        let raw = r#"{"type":"key_items_held","game_character_id":12345,"key_item_ids":[1,42]}"#;
        assert_eq!(
            parse_message(raw).unwrap(),
            AddonMessage::KeyItemsHeld { game_character_id: 12345, key_item_ids: vec![1, 42] }
        );
    }

    #[test]
    fn parses_key_items_held_message_with_empty_key_item_ids() {
        let raw = r#"{"type":"key_items_held","game_character_id":12345,"key_item_ids":[]}"#;
        assert_eq!(
            parse_message(raw).unwrap(),
            AddonMessage::KeyItemsHeld { game_character_id: 12345, key_item_ids: vec![] }
        );
    }
}
