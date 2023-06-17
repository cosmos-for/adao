use std::collections::HashSet;

use cosmwasm_std::{
    coins, BankMsg, DepsMut, Empty, Event, MessageInfo, Order, Response, StdResult,
};

use crate::{
    error::ContractError,
    state::{ADMINS, DONATION_DENOM},
};

pub fn add_members(
    deps: DepsMut,
    info: MessageInfo,
    admins: Vec<String>,
) -> Result<Response, ContractError> {
    // let mut curr_admins = ADMINS.load(deps.storage)?;
    // let admins: Result<Vec<_>, _> = ADMINS
    //     .keys(deps.storage, None, None, Order::Ascending)
    //     .collect();
    // let admin = admins?;

    if !ADMINS.has(deps.storage, &info.sender) {
        return Err(ContractError::Unauthorized {
            sender: info.sender,
        });
    }

    let admins: HashSet<String> = HashSet::from_iter(admins.into_iter());

    let events = admins
        .iter()
        // the event key will has a prefix "wasm-" in event, e.g. the key is `wasm-admin_added` here
        .map(|admin| Event::new("admin_added").add_attribute("addr", admin));

    let resp = Response::new()
        .add_events(events)
        // the event key will be "wasm" in attribute, e.g. the key is `wasm` here
        .add_attribute("action", "add_members")
        .add_attribute("added_count", admins.len().to_string());

    for addr in &admins {
        let admin = deps.api.addr_validate(addr)?;
        if !ADMINS.has(deps.storage, &admin) {
            ADMINS.save(deps.storage, &admin, &Empty {})?;
        }
    }

    // let admins: StdResult<Vec<_>> = admins
    //     .into_iter()
    //     .map(|addr| deps.api.addr_validate(&addr))
    //     .collect();

    // curr_admins.append(&mut admins?);

    // ADMINS.save(deps.storage, &curr_admins)?;

    Ok(resp)
}

pub fn leave(deps: DepsMut, info: MessageInfo) -> StdResult<Response> {
    ADMINS.remove(deps.storage, &info.sender);

    let resp = Response::new()
        .add_attribute("action", "leave")
        .add_attribute("sender", info.sender.as_str());

    Ok(resp)
}

pub fn donate(deps: DepsMut, info: MessageInfo) -> Result<Response, ContractError> {
    let denom = DONATION_DENOM.load(deps.storage)?;
    let admins: Result<Vec<_>, _> = ADMINS
        .keys(deps.storage, None, None, Order::Ascending)
        .collect();

    let admins = admins?;

    let donation = cw_utils::must_pay(&info, &denom)?.u128();
    let donation_per_admin = donation / (admins.len() as u128);

    let messages = admins.into_iter().map(|admin| BankMsg::Send {
        to_address: admin.to_string(),
        amount: coins(donation_per_admin, &denom),
    });

    let resp = Response::new()
        .add_messages(messages)
        .add_attribute("action", "donate")
        .add_attribute("donated_amount", donation.to_string())
        .add_attribute("donation_per_admin", donation_per_admin.to_string());

    Ok(resp)
}
