use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::{Addr, Deps, StdResult};

use crate::state::ADMINS;

#[cw_serde]
#[derive(Default)]
pub struct InstantiateMsg {
    pub admins: Vec<String>,
    pub donation_denom: String,
}

impl InstantiateMsg {
    pub fn new(admins: Vec<String>, donation_denom: impl Into<String>) -> Self {
        Self {
            admins,
            donation_denom: donation_denom.into(),
        }
    }
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(AdminsListResp)]
    AdminsList {},
}

#[cw_serde]
#[derive(Default)]
pub struct AdminsListResp {
    pub admins: Vec<Addr>,
}

impl AdminsListResp {
    #[allow(dead_code)]
    pub fn new(admins: Vec<Addr>) -> Self {
        Self { admins }
    }
}

#[cw_serde]
pub enum ExecuteMsg {
    AddMemebers { admins: Vec<String> },
    Leave {},
    Donate {},
}

pub fn admins_list(deps: Deps) -> StdResult<AdminsListResp> {
    let admins = ADMINS.load(deps.storage)?;
    let resp = AdminsListResp { admins };
    Ok(resp)
}
