pub mod cosmos;
pub mod evm;
pub mod carbon;
pub mod fee;
pub mod carbon_tx;

pub fn strip_quotes(input: &str) -> &str {
    input.trim_matches('"')
}