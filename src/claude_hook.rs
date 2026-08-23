use serde_json::Value;

use crate::model::{Client, Initiator, UsageEvent};

pub fn normalize(payload: &[u8]) -> Result<Option<UsageEvent>, serde_json::Error> {
    let payload: Value = serde_json::from_slice(payload)?;
    let Some(object) = payload.as_object() else {
        return Ok(None);
    };

    let event = match object.get("hook_event_name").and_then(Value::as_str) {
        Some("UserPromptExpansion") => normalize_user(object),
        Some("PostToolUse") => normalize_model(object),
        _ => None,
    };

    Ok(event)
}

fn normalize_user(object: &serde_json::Map<String, Value>) -> Option<UsageEvent> {
    if object.get("expansion_type").and_then(Value::as_str) != Some("slash_command") {
        return None;
    }

    let skill = object.get("command_name")?.as_str()?.to_owned();
    UsageEvent::observed(Client::ClaudeCode, skill, Initiator::User)
}

fn normalize_model(object: &serde_json::Map<String, Value>) -> Option<UsageEvent> {
    if object.get("tool_name").and_then(Value::as_str) != Some("Skill") {
        return None;
    }

    let input_name = object
        .get("tool_input")?
        .as_object()?
        .get("skill")?
        .as_str()?;
    let response = object.get("tool_response")?.as_object()?;
    if response.get("success").and_then(Value::as_bool) != Some(true) {
        return None;
    }
    let response_name = response.get("commandName")?.as_str()?;
    if input_name != response_name {
        return None;
    }

    UsageEvent::observed(
        Client::ClaudeCode,
        response_name.to_owned(),
        Initiator::Model,
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use uuid::Uuid;

    use crate::storage::Store;

    #[test]
    fn accepts_user_expansion_without_retaining_payload_fields() {
        let event = normalize(
            br#"{
                "hook_event_name":"UserPromptExpansion",
                "expansion_type":"slash_command",
                "command_name":"Plugin:Review",
                "prompt":"PRIVATE CANARY",
                "cwd":"/private/canary"
            }"#,
        )
        .unwrap()
        .unwrap();

        assert_eq!(event.client, Client::ClaudeCode);
        assert_eq!(event.initiator, Initiator::User);
        assert_eq!(event.skill, "Plugin:Review");
    }

    #[test]
    fn accepts_only_consistent_successful_model_results() {
        let valid = br#"{
            "hook_event_name":"PostToolUse",
            "tool_name":"Skill",
            "tool_input":{"skill":"tdd","args":"PRIVATE CANARY"},
            "tool_response":{"commandName":"tdd","success":true},
            "tool_use_id":"secret"
        }"#;
        assert_eq!(normalize(valid).unwrap().unwrap().skill, "tdd");

        for invalid in [
            br#"{"hook_event_name":"PostToolUse","tool_name":"Skill","tool_input":{"skill":"tdd"},"tool_response":{"commandName":"other","success":true}}"#.as_slice(),
            br#"{"hook_event_name":"PostToolUse","tool_name":"Skill","tool_input":{"skill":"tdd"},"tool_response":{"commandName":"tdd","success":false}}"#.as_slice(),
            br#"{"hook_event_name":"PreToolUse","tool_name":"Skill","tool_input":{"skill":"tdd"}}"#.as_slice(),
        ] {
            assert!(normalize(invalid).unwrap().is_none());
        }
    }

    #[test]
    fn raw_hook_fields_never_reach_sqlite() {
        const CANARY: &str = "agentlens-private-prompt-canary-7ee5";
        let payload = format!(
            r#"{{
                "hook_event_name":"UserPromptExpansion",
                "expansion_type":"slash_command",
                "command_name":"safe-skill",
                "prompt":"{CANARY}",
                "tool_input":{{"secret":"{CANARY}"}}
            }}"#
        );
        let event = normalize(payload.as_bytes()).unwrap().unwrap();
        let path =
            std::env::temp_dir().join(format!("agentlens-privacy-{}.sqlite3", Uuid::now_v7()));
        let mut store = Store::open(&path).unwrap();
        store.insert(&event).unwrap();
        store
            .connection()
            .execute_batch("PRAGMA wal_checkpoint(TRUNCATE)")
            .unwrap();
        drop(store);

        let database = std::fs::read(&path).unwrap();
        assert!(!String::from_utf8_lossy(&database).contains(CANARY));

        let _ = std::fs::remove_file(&path);
        let _ = std::fs::remove_file(path.with_extension("sqlite3-shm"));
        let _ = std::fs::remove_file(path.with_extension("sqlite3-wal"));
    }
}
