use cosmwasm_std::StdResult;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub enum QueryMsg {
    Greet {},
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct GreetResp {
    pub message: String,
}

impl GreetResp {
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }
}

pub fn greet() -> StdResult<GreetResp> {
    let resp = GreetResp::new("hello world!");

    Ok(resp)
}

#[cfg(test)]
mod tests {

    // use crate::msg::GreetResp;

    use super::*;

    #[test]
    fn greet_test() {
        let resp = greet().unwrap();

        assert_eq!(resp, GreetResp::new("hello world!"))
    }
}
