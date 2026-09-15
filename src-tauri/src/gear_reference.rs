use crate::db::GearSetDefinitionRow;
use serde::Deserialize;

/// Windower job ids 1-22, in the canonical order `src/jobs.ts`'s JOBS array
/// and `tools/scraper/config.ts`'s CANONICAL_JOBS already share (confirmed
/// 1:1 by index during spec review) — index 0 here = job_id 1.
const JOB_NAMES: [&str; 22] = [
    "Warrior", "Monk", "White Mage", "Black Mage", "Red Mage", "Thief", "Paladin",
    "Dark Knight", "Beastmaster", "Bard", "Ranger", "Samurai", "Ninja", "Dragoon",
    "Summoner", "Blue Mage", "Corsair", "Puppetmaster", "Dancer", "Scholar",
    "Geomancer", "Rune Fencer",
];

pub fn job_id_for_name(name: &str) -> Option<i64> {
    JOB_NAMES.iter().position(|&n| n == name).map(|i| i as i64 + 1)
}

#[derive(Debug, Deserialize)]
struct ScrapedGearEntry {
    job: String,
    #[serde(rename = "setType")]
    set_type: String,
    slot: String,
    tier: String,
    #[serde(rename = "itemName")]
    item_name: String,
    #[serde(rename = "itemId")]
    item_id: Option<i64>,
}

/// Panics on an unknown job name or non-numeric tier: this only ever runs
/// against the bundled (compile-time-embedded) scraper JSON, whose job
/// names are guaranteed to come from the same 22-entry list by construction
/// (tools/scraper/config.ts's CANONICAL_JOBS) — a mismatch here means the
/// bundled reference data itself is corrupt, not a normal runtime condition
/// worth a recoverable Result for.
pub fn parse_gear_set_definitions(json: &str) -> serde_json::Result<Vec<GearSetDefinitionRow>> {
    let entries: Vec<ScrapedGearEntry> = serde_json::from_str(json)?;
    Ok(entries
        .into_iter()
        .map(|e| GearSetDefinitionRow {
            job_id: job_id_for_name(&e.job)
                .unwrap_or_else(|| panic!("unknown job name '{}' in bundled gear set data", e.job)),
            set_type: e.set_type,
            slot: e.slot,
            tier: e.tier.parse().unwrap_or_else(|_| {
                panic!("non-numeric tier '{}' for item '{}'", e.tier, e.item_name)
            }),
            item_name: e.item_name,
            item_id: e.item_id,
        })
        .collect())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn job_id_for_name_covers_all_22_canonical_jobs() {
        let expected: [(&str, i64); 22] = [
            ("Warrior", 1), ("Monk", 2), ("White Mage", 3), ("Black Mage", 4), ("Red Mage", 5),
            ("Thief", 6), ("Paladin", 7), ("Dark Knight", 8), ("Beastmaster", 9), ("Bard", 10),
            ("Ranger", 11), ("Samurai", 12), ("Ninja", 13), ("Dragoon", 14), ("Summoner", 15),
            ("Blue Mage", 16), ("Corsair", 17), ("Puppetmaster", 18), ("Dancer", 19),
            ("Scholar", 20), ("Geomancer", 21), ("Rune Fencer", 22),
        ];
        for (name, id) in expected {
            assert_eq!(job_id_for_name(name), Some(id), "job name: {name}");
        }
    }

    #[test]
    fn job_id_for_name_returns_none_for_unknown_name() {
        assert_eq!(job_id_for_name("Not A Job"), None);
    }

    #[test]
    fn parse_gear_set_definitions_translates_job_name_and_tier() {
        let json = r#"[{"job":"Scholar","setType":"af3","slot":"head","tier":"0","itemName":"Academic's Mortarboard","itemId":27683}]"#;
        let rows = parse_gear_set_definitions(json).unwrap();
        assert_eq!(rows, vec![GearSetDefinitionRow {
            job_id: 20,
            set_type: "af3".to_string(),
            slot: "head".to_string(),
            tier: 0,
            item_name: "Academic's Mortarboard".to_string(),
            item_id: Some(27683),
        }]);
    }

    #[test]
    fn parse_gear_set_definitions_keeps_null_item_id_rows() {
        let json = r#"[{"job":"Scholar","setType":"relic","slot":"head","tier":"2","itemName":"Argute Mortarboard +2","itemId":null}]"#;
        let rows = parse_gear_set_definitions(json).unwrap();
        assert_eq!(rows[0].item_id, None);
    }

    #[test]
    fn parse_gear_set_definitions_rejects_malformed_json() {
        assert!(parse_gear_set_definitions("not json").is_err());
    }

    #[test]
    fn parses_real_bundled_gear_set_files_without_panicking() {
        for json in [
            include_str!("../../tools/scraper/out/gearsets-af3.json"),
            include_str!("../../tools/scraper/out/gearsets-empyrean.json"),
            include_str!("../../tools/scraper/out/gearsets-relic.json"),
        ] {
            parse_gear_set_definitions(json).unwrap();
        }
    }
}
