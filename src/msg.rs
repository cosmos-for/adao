use cosmwasm_std::{Addr, Deps, StdResult};
use serde::{Deserialize, Serialize};

use crate::state::ADMINS;

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Default)]
pub struct InstantiateMsg {
    pub admins: Vec<String>,
}

impl InstantiateMsg {
    pub fn new(admins: Vec<String>) -> Self {
        Self { admins }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub enum QueryMsg {
    Greet {},
    AdminsList {},
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Default)]
pub struct AdminsListResp {
    pub admins: Vec<Addr>,
}

impl AdminsListResp {
    #[allow(dead_code)]
    pub fn new(admins: Vec<Addr>) -> Self {
        Self { admins }
    }
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

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub enum ExecuteMsg {
    AddMemebers { admins: Vec<String> },
    Leave {},
}

pub fn admins_list(deps: Deps) -> StdResult<AdminsListResp> {
    let admins = ADMINS.load(deps.storage)?;
    let resp = AdminsListResp { admins };
    Ok(resp)
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
