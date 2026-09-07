use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum AddonMessage {
    Handshake { game_character_id: i64, name: String },
    Heartbeat { game_character_id: i64 },
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
}
