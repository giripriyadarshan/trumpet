use std::collections::HashSet;

pub fn convert_string_to_set(string: String) -> HashSet<String> {
    string
        .split(", ")
        .map(|s| s.to_string())
        .collect::<HashSet<String>>()
}

pub fn convert_set_to_string(set: HashSet<String>) -> String {
    set.into_iter().collect::<Vec<String>>().join(", ")
}
