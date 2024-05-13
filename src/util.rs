pub mod cosmos;

pub fn strip_quotes(input: &str) -> &str {
    input.trim_matches('"')
}