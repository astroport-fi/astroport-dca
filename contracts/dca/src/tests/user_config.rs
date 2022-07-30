use std::error::Error;

use astroport_dca::{ExecuteMsg, QueryMsg, UserConfig};
use cosmwasm_std::{Addr, Decimal};
use cw_multi_test::Executor;

use super::common::{instantiate, USER_ONE};

#[test]
fn update_user_config_works() -> Result<(), Box<dyn Error>> {
    let (mut app, dca) = instantiate();

    app.execute_contract(
        Addr::unchecked(USER_ONE),
        dca.clone(),
        &ExecuteMsg::UpdateUserConfig {
            max_hops: Some(1),
            max_spread: Some(Decimal::percent(2)),
        },
        &[],
    )?;

    let UserConfig {
        max_hops,
        max_spread,
        ..
    } = app.wrap().query_wasm_smart(
        &dca,
        &QueryMsg::UserConfig {
            user: USER_ONE.to_string(),
        },
    )?;

    assert_eq!(max_hops, Some(1));
    assert_eq!(max_spread, Some(Decimal::percent(2)));

    Ok(())
}
