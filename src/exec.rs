use cosmwasm_std::{DepsMut, MessageInfo, Response, StdResult};

use crate::{error::ContractError, state::ADMINS};

pub fn add_members(
    deps: DepsMut,
    info: MessageInfo,
    admins: Vec<String>,
) -> Result<Response, ContractError> {
    let mut curr_admins = ADMINS.load(deps.storage)?;

    if !curr_admins.contains(&info.sender) {
        return Err(ContractError::Unauthorized {
            sender: info.sender,
        });
    }

    let admins: StdResult<Vec<_>> = admins
        .into_iter()
        .map(|addr| deps.api.addr_validate(&addr))
        .collect();

    curr_admins.append(&mut admins?);

    ADMINS.save(deps.storage, &curr_admins)?;

    Ok(Response::new())
}

pub fn leave(deps: DepsMut, info: MessageInfo) -> StdResult<Response> {
    ADMINS.update(deps.storage, move |admins| -> StdResult<_> {
        let admins = admins
            .into_iter()
            .filter(|addr| *addr != info.sender)
            .collect();

        Ok(admins)
    })?;

    Ok(Response::new())
}
