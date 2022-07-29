use astroport::asset::UUSD_DENOM;
use astroport_dca::UserConfig;
use cosmwasm_std::{attr, DepsMut, MessageInfo, Response, StdResult};

use crate::{error::ContractError, state::USER_CONFIG};

/// ## Description
/// Adds a tip to the contract for a users DCA purchases.
///
/// Returns a [`ContractError`] as a failure, otherwise returns a [`Response`] with the specified
/// attributes if the operation was successful.
/// ## Arguments
/// * `deps` - A [`DepsMut`] that contains the dependencies.
///
/// * `info` - A [`MessageInfo`] which contains a uusd tip to add to a users tip balance.
pub fn add_bot_tip(deps: DepsMut, info: MessageInfo) -> Result<Response, ContractError> {
    let amount = info
        .funds
        .iter()
        .find(|coin| coin.denom == UUSD_DENOM)
        .ok_or(ContractError::InvalidZeroAmount {})?
        .amount;

    // update user tip in contract
    USER_CONFIG.update(
        deps.storage,
        &info.sender,
        |config| -> StdResult<UserConfig> {
            let mut config = config.unwrap_or_default();

            config.tip_balance = config.tip_balance.checked_add(amount)?;

            Ok(config)
        },
    )?;

    Ok(Response::new().add_attributes(vec![
        attr("action", "add_bot_tip"),
        attr("tip_amount", amount),
    ]))
}

#[cfg(test)]
mod tests {
    use astroport_dca::{ExecuteMsg, UserConfig};
    use cosmwasm_std::{
        attr, coin,
        testing::{mock_dependencies, mock_env, mock_info},
        Addr, Response,
    };

    use crate::{contract::execute, error::ContractError, state::USER_CONFIG};

    #[test]
    fn does_add_bot_tip() {
        let mut deps = mock_dependencies();

        let tip_sent = coin(10000, "uusd");

        let info = mock_info("creator", &[tip_sent.clone()]);
        let msg = ExecuteMsg::AddBotTip {};

        // check that we got the expected response
        let res = execute(deps.as_mut(), mock_env(), info, msg).unwrap();
        assert_eq!(
            res,
            Response::new().add_attributes(vec![
                attr("action", "add_bot_tip"),
                attr("tip_amount", tip_sent.amount)
            ])
        );

        // check that user tip balance was added
        let config = USER_CONFIG
            .load(&deps.storage, &Addr::unchecked("creator"))
            .unwrap();
        assert_eq!(
            config,
            UserConfig {
                tip_balance: tip_sent.amount,
                ..UserConfig::default()
            }
        )
    }

    #[test]
    fn does_require_funds() {
        let mut deps = mock_dependencies();

        let info = mock_info("creator", &[]);
        let msg = ExecuteMsg::AddBotTip {};

        // should error with InvalidZeroAmount failure
        let res = execute(deps.as_mut(), mock_env(), info, msg).unwrap_err();
        assert_eq!(res, ContractError::InvalidZeroAmount {});
    }

    #[test]
    fn does_require_uusd_funds() {
        let mut deps = mock_dependencies();

        let info = mock_info("creator", &[coin(20000, "ukrw")]);
        let msg = ExecuteMsg::AddBotTip {};

        // should error with InvalidZeroAmount
        let res = execute(deps.as_mut(), mock_env(), info, msg).unwrap_err();
        assert_eq!(res, ContractError::InvalidZeroAmount {});
    }
}
