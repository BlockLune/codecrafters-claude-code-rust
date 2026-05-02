pub fn get_tools() -> Vec<serde_json::Value> {
    vec![serde_json::from_str(include_str!("./tools/read.json")).unwrap()]
}
