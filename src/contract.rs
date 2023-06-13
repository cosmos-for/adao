use crate::msg::{self, QueryMsg, InstantiateMsg};
use cosmwasm_std::{
    to_binary, Binary, Deps, DepsMut, Empty, Env, MessageInfo, Response, StdResult,
};

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

    Ok(Response::new())
}

pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    use QueryMsg::*;

    match msg {
        Greet {} => to_binary(&msg::greet()?),
        AdminsList {  } => to_binary(&msg::admins_list(deps)?),
    }
}

#[allow(dead_code)]
pub fn execute(
    _deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    _msg: Empty,
) -> StdResult<Response> {
    unimplemented!()
}

#[cfg(test)]
mod tests {
    use cosmwasm_std::{from_binary, Addr};
    use cosmwasm_std::testing::{mock_dependencies, mock_env, mock_info};
    use cw_multi_test::{App, ContractWrapper, Executor};

    use crate::contract::instantiate;
    use crate::msg::{GreetResp, QueryMsg, InstantiateMsg, AdminsListResp};

    use super::{query, execute};

    #[test]
    fn greet_query_test() {
        let mut deps = mock_dependencies();
        let env = mock_env();

        instantiate(deps.as_mut(), env.clone(), mock_info("sender", &[]), InstantiateMsg::default())
            .unwrap();
        
        let resp = query(deps.as_ref(), env, QueryMsg::Greet { }).unwrap();

        let resp: GreetResp = from_binary(&resp).unwrap();

        assert_eq!(resp, GreetResp::new("hello world!"),)
    }

    #[test]
    fn contract_multi_tests_work() {
        let mut app = App::default();
        
        let code = ContractWrapper::new(execute, instantiate, query);
        let code_id = app.store_code(Box::new(code));

        let addr = app
            .instantiate_contract(code_id, Addr::unchecked("owner"), &InstantiateMsg::default(), &[], "Contract", None)
            .unwrap();

        let resp: GreetResp = app
            .wrap()
            .query_wasm_smart(addr, &QueryMsg::Greet {})
            .unwrap();

        assert_eq!(
            resp,
            GreetResp::new("hello world!"),
        );
    }

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
            .query_wasm_smart(addr, &QueryMsg::AdminsList {  })
            .unwrap();

        assert_eq!(
            resp,
            AdminsListResp::default(),
        );

        let addr = app
            .instantiate_contract(
                code_id, 
                Addr::unchecked("owner"),
                &InstantiateMsg::new(vec!["admin1".to_string(), "admin2".to_string()]),
                &[],
                "Contract2",
                None,
            )
            .unwrap();
    
        let resp: AdminsListResp = app
            .wrap()
            .query_wasm_smart(addr, &QueryMsg::AdminsList {  })
            .unwrap();

        assert_eq!(
            resp,
            AdminsListResp::new(vec![Addr::unchecked("admin1"), Addr::unchecked("admin2")]),
        )

    }
}
