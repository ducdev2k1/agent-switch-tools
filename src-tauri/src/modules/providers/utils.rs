pub fn extract_plan_from_proto_json(json: &serde_json::Value) -> Option<String> {
    let b64 = json.get("userStatusProtoBinaryBase64")?.as_str()?;
    let decoded = data_encoding::BASE64.decode(b64.as_bytes()).ok()?;
    let text = String::from_utf8_lossy(&decoded);
    let known_plans = ["Enterprise", "Team", "Pro Ultimate", "Pro", "Free"];
    for plan in &known_plans {
        if text.contains(plan) {
            return Some(plan.to_string());
        }
    }
    None
}
