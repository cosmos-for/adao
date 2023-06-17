use crate::{
    error::ContractError,
    exec,
    msg::{self, ExecuteMsg, InstantiateMsg, QueryMsg},
    state::DONATION_DENOM,
};
use cosmwasm_std::{to_binary, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult};

use crate::state::ADMINS;

pub fn instantiate(
    deps: DepsMut,
    env: Env,
    _info: MessageInfo,
    msg: InstantiateMsg,
) -> StdResult<Response> {
    for addr in msg.admins {
        let admin = deps.api.addr_validate(&addr)?;
        ADMINS.save(deps.storage, &admin, &env.block.time)?;
    }

    DONATION_DENOM.save(deps.storage, &msg.donation_denom)?;

    Ok(Response::new())
}

pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    use QueryMsg::*;

    match msg {
        AdminsList {} => to_binary(&msg::admins_list(deps)?),
        JoinTime { admin } => to_binary(&msg::join_time(deps, admin)?),
    }
}

pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    use ExecuteMsg::*;

    match msg {
        AddMemebers { admins } => exec::add_members(deps, info, admins, &env.block.time),
        Leave {} => exec::leave(deps, info).map_err(Into::into),
        Donate {} => exec::donate(deps, info),
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;

    use cosmwasm_std::{coins, Addr};
    use cw_multi_test::{App, ContractWrapper, Executor};

    use crate::contract::instantiate;
    use crate::error::ContractError;
    use crate::msg::{AdminsListResp, ExecuteMsg, InstantiateMsg, JoinTimeResp, QueryMsg};

    use super::{execute, query};

    const OWNER: &str = "cosmos1z046yq9fzdf4ll4gkqg3pzgzguzscfmrjevdz2";
    const USER: &str = "cosmos10uvqq6fd7aqn5yq3aczha3r0956yecxmmj3aev";
    const ADMIN1: &str = "cosmos1kajjl5f959kdf4hdjpa0r8cuvg6au5rr6rxrl6";
    const ADMIN2: &str = "cosmos1cdn6hcs76r548k5a672zd9aueff9rukeupzdpf";

    #[test]
    fn instantiate_test() {
        let mut app = App::default();

        let code = ContractWrapper::new(execute, instantiate, query);
        let code_id = app.store_code(Box::new(code));

        let addr = app
            .instantiate_contract(
                code_id,
                Addr::unchecked(OWNER),
                &InstantiateMsg::default(),
                &[],
                "Contract",
                None,
            )
            .unwrap();

        let resp: AdminsListResp = app
            .wrap()
            .query_wasm_smart(addr, &QueryMsg::AdminsList {})
            .unwrap();

        assert_eq!(resp, AdminsListResp::default(),);

        let addr = app
            .instantiate_contract(
                code_id,
                Addr::unchecked(OWNER),
                &InstantiateMsg::new(vec![ADMIN1.to_string(), ADMIN2.to_string()], "wasm"),
                &[],
                "Contract2",
                None,
            )
            .unwrap();

        let resp: AdminsListResp = app
            .wrap()
            .query_wasm_smart(addr, &QueryMsg::AdminsList {})
            .unwrap();

        assert_eq!(
            BTreeSet::from_iter(resp.admins.iter()),
            BTreeSet::from_iter(
                AdminsListResp::new(vec![Addr::unchecked(ADMIN1), Addr::unchecked(ADMIN2)])
                    .admins
                    .iter()
            ),
        )
    }

    #[test]
    fn add_members_test() {
        let mut app = App::default();

        let code = ContractWrapper::new(execute, instantiate, query);
        let code_id = app.store_code(Box::new(code));

        let addr = app
            .instantiate_contract(
                code_id,
                Addr::unchecked(OWNER),
                &InstantiateMsg::new(vec![OWNER.to_owned()], "wasm"),
                &[],
                "Contract",
                None,
            )
            .unwrap();

        let resp = app
            .execute_contract(
                Addr::unchecked(OWNER),
                addr.clone(),
                &ExecuteMsg::AddMemebers {
                    admins: vec![USER.to_owned()],
                },
                &[],
            )
            .unwrap();

        let wasm_events = resp.events.iter().find(|e| e.ty == "wasm").unwrap();

        assert_eq!(
            wasm_events
                .attributes
                .iter()
                .find(|a| a.key == "action")
                .unwrap()
                .value,
            "add_members"
        );

        assert_eq!(
            wasm_events
                .attributes
                .iter()
                .find(|a| a.key == "added_count")
                .unwrap()
                .value,
            "1"
        );

        let admin_added_events: Vec<_> = resp
            .events
            .iter()
            .filter(|ev| ev.ty == "wasm-admin_added")
            .collect();

        assert_eq!(admin_added_events.len(), 1);

        assert_eq!(
            admin_added_events[0]
                .attributes
                .iter()
                .find(|a| a.key == "addr")
                .unwrap()
                .value,
            USER,
        );

        let join_time: JoinTimeResp = app
            .wrap()
            .query_wasm_smart(
                addr,
                &QueryMsg::JoinTime {
                    admin: USER.to_owned(),
                },
            )
            .unwrap();

        let now = app.block_info().time;

        assert_eq!(join_time.joined, now);
    }

    #[test]
    fn unauthorized_test() {
        let mut app = App::default();

        let code = ContractWrapper::new(execute, instantiate, query);
        let code_id = app.store_code(Box::new(code));

        let addr = app
            .instantiate_contract(
                code_id,
                Addr::unchecked(OWNER),
                &InstantiateMsg::new(vec![OWNER.to_owned()], "wasm"),
                &[],
                "Contract",
                None,
            )
            .unwrap();

        let err = app
            .execute_contract(
                Addr::unchecked(USER),
                addr,
                &ExecuteMsg::AddMemebers {
                    admins: vec![USER.to_string()],
                },
                &[],
            )
            .unwrap_err();

        assert_eq!(
            ContractError::Unauthorized {
                sender: Addr::unchecked(USER)
            },
            err.downcast().unwrap()
        )
    }

    #[test]
    fn donate_test() {
        let mut app = App::new(|router, _, storage| {
            router
                .bank
                .init_balance(storage, &Addr::unchecked(USER), coins(5, "eth"))
                .unwrap()
        });

        let code = ContractWrapper::new(execute, instantiate, query);
        let code_id = app.store_code(Box::new(code));

        let addr = app
            .instantiate_contract(
                code_id,
                Addr::unchecked(OWNER.to_owned()),
                &InstantiateMsg::new(vec![ADMIN1.to_owned(), ADMIN2.to_owned()], "eth"),
                &[],
                "Contract",
                None,
            )
            .unwrap();

        app.execute_contract(
            Addr::unchecked(USER),
            addr.clone(),
            &ExecuteMsg::Donate {},
            &coins(3, "eth"),
        )
        .unwrap();

        assert_eq!(get_balance(&app, USER, "eth"), 2,);

        assert_eq!(get_balance(&app, ADMIN1, "eth"), 1,);

        assert_eq!(get_balance(&app, ADMIN2, "eth"), 1,);

        assert_eq!(get_balance(&app, addr.as_str(), "eth"), 1,);
    }

    fn get_balance(app: &App, addr: &str, denom: &str) -> u128 {
        app.wrap().query_balance(addr, denom).unwrap().amount.u128()
    }
}
