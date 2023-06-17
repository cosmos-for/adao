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
    _env: Env,
    _info: MessageInfo,
    msg: InstantiateMsg,
) -> StdResult<Response> {
    let admins: StdResult<Vec<_>> = msg
        .admins
        .into_iter()
        .map(|addr| deps.api.addr_validate(&addr))
        .collect();

    ADMINS.save(deps.storage, &admins?)?;
    DONATION_DENOM.save(deps.storage, &msg.donation_denom)?;

    Ok(Response::new())
}

pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    use QueryMsg::*;

    match msg {
        AdminsList {} => to_binary(&msg::admins_list(deps)?),
    }
}

pub fn execute(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    use ExecuteMsg::*;

    match msg {
        AddMemebers { admins } => exec::add_members(deps, info, admins),
        Leave {} => exec::leave(deps, info).map_err(Into::into),
        Donate {} => exec::donate(deps, info),
    }
}

#[cfg(test)]
mod tests {
    use cosmwasm_std::{coins, Addr};
    use cw_multi_test::{App, ContractWrapper, Executor};

    use crate::contract::instantiate;
    use crate::error::ContractError;
    use crate::msg::{AdminsListResp, ExecuteMsg, InstantiateMsg, QueryMsg};

    use super::{execute, query};

    #[test]
    fn instantiate_test() {
        let mut app = App::default();

        let code = ContractWrapper::new(execute, instantiate, query);
        let code_id = app.store_code(Box::new(code));

        let addr = app
            .instantiate_contract(
                code_id,
                Addr::unchecked("owner"),
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
                Addr::unchecked("owner"),
                &InstantiateMsg::new(vec!["admin1".to_string(), "admin2".to_string()], "wasm"),
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
            resp,
            AdminsListResp::new(vec![Addr::unchecked("admin1"), Addr::unchecked("admin2")]),
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
                Addr::unchecked("owner"),
                &InstantiateMsg::new(vec!["owner".to_owned()], "wasm"),
                &[],
                "Contract",
                None,
            )
            .unwrap();

        let resp = app
            .execute_contract(
                Addr::unchecked("owner"),
                addr,
                &ExecuteMsg::AddMemebers {
                    admins: vec!["user".to_owned()],
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
            "user",
        );
    }

    #[test]
    fn unauthorized_test() {
        let mut app = App::default();

        let code = ContractWrapper::new(execute, instantiate, query);
        let code_id = app.store_code(Box::new(code));

        let addr = app
            .instantiate_contract(
                code_id,
                Addr::unchecked("owner"),
                &InstantiateMsg::new(vec!["owner".to_owned()], "wasm"),
                &[],
                "Contract",
                None,
            )
            .unwrap();

        let err = app
            .execute_contract(
                Addr::unchecked("user"),
                addr,
                &ExecuteMsg::AddMemebers {
                    admins: vec!["user".to_string()],
                },
                &[],
            )
            .unwrap_err();

        assert_eq!(
            ContractError::Unauthorized {
                sender: Addr::unchecked("user")
            },
            err.downcast().unwrap()
        )
    }

    #[test]
    fn donate_test() {
        let mut app = App::new(|router, _, storage| {
            router
                .bank
                .init_balance(storage, &Addr::unchecked("user"), coins(5, "eth"))
                .unwrap()
        });

        let code = ContractWrapper::new(execute, instantiate, query);
        let code_id = app.store_code(Box::new(code));

        let addr = app
            .instantiate_contract(
                code_id,
                Addr::unchecked("owner".to_owned()),
                &InstantiateMsg::new(vec!["admin1".to_owned(), "admin2".to_owned()], "eth"),
                &[],
                "Contract",
                None,
            )
            .unwrap();

        app.execute_contract(
            Addr::unchecked("user"),
            addr.clone(),
            &ExecuteMsg::Donate {},
            &coins(3, "eth"),
        )
        .unwrap();

        assert_eq!(get_balance(&app, "user", "eth"), 2,);

        assert_eq!(get_balance(&app, "admin1", "eth"), 1,);

        assert_eq!(get_balance(&app, "admin2", "eth"), 1,);

        assert_eq!(get_balance(&app, addr.as_str(), "eth"), 1,);
    }

    fn get_balance(app: &App, addr: &str, denom: &str) -> u128 {
        app.wrap().query_balance(addr, denom).unwrap().amount.u128()
    }
}
