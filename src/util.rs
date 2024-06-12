pub mod cosmos;
pub mod evm;
pub mod carbon;
pub mod carbon_tx;
pub mod carbon_msg;
pub mod datetime;

pub fn strip_quotes(input: &str) -> &str {
    input.trim_matches('"')
}