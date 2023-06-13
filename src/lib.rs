mod contract;

mod msg;

mod state;

use cosmwasm_std::{
    entry_point, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult,
};
use msg::{QueryMsg, InstantiateMsg};

#[entry_point]
pub fn instantiate(deps: DepsMut, env: Env, info: MessageInfo, msg: InstantiateMsg) -> StdResult<Response> {
    contract::instantiate(deps, env, info, msg)
}

#[entry_point]
pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> StdResult<Binary> {
    contract::query(deps, env, msg)
}
