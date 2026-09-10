use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub struct JobEntry {
    pub job_id: i64,
    pub level: i64,
    pub master_level: i64,
    pub mastered: bool,
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
}
