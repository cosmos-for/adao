use cosmwasm_schema::write_api;

use contract::msg::{InstantiateMsg, ExecuteMsg, QueryMsg};


fn main() {
    write_api! {
        instantiate: InstantiateMsg,
        execute:ExecuteMsg,
        query: QueryMsg,
    }
}