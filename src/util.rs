use regex::Regex;

pub fn unquote_parsed_string_value(value: &str) -> String {
    Regex::new("^(?:<|\")|(?:>|\")$")
        .unwrap()
        .replace_all(value, "")
        .to_string()
}
