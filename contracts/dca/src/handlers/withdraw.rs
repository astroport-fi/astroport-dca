use astroport::asset::UUSD_DENOM;
use cosmwasm_std::{attr, coins, BankMsg, DepsMut, MessageInfo, Response, Uint128};

use crate::{error::ContractError, state::USER_CONFIG};

/// ## Description
/// Withdraws a users bot tip from the contract.
///
/// Returns a [`ContractError`] as a failure, otherwise returns a [`Response`] with the specified
/// attributes if the operation was successful.
/// ## Arguments
/// * `deps` - A [`DepsMut`] that contains the dependencies.
///
/// * `info` - A [`MessageInfo`] from the sender who wants to withdraw their bot tip.
///
/// * `amount`` - A [`Uint128`] representing the amount of uusd to send back to the user.
pub fn withdraw(
    deps: DepsMut,
    info: MessageInfo,
    amount: Uint128,
) -> Result<Response, ContractError> {
    let mut config = USER_CONFIG
        .may_load(deps.storage, &info.sender)?
        .unwrap_or_default();

    config.tip_balance = config.tip_balance.checked_sub(amount)?;

    // optimization: if the user is withdrawing all their tip, they are probably never going to
    // interact with the contract again. In this case, we can delete their config to save space
    // otherwise, we save their new configuration
    match config.tip_balance.is_zero() {
        true => {
            USER_CONFIG.remove(deps.storage, &info.sender);
            Ok(())
        }
        false => USER_CONFIG.save(deps.storage, &info.sender, &config),
    }?;

    Ok(Response::new()
        .add_attributes(vec![
            attr("action", "withdraw"),
            attr("tip_removed", amount),
        ])
        .add_message(BankMsg::Send {
            to_address: info.sender.to_string(),
            amount: coins(amount.u128(), UUSD_DENOM),
        }))
}

#[cfg(test)]
mod tests {
    use astroport_dca::{ExecuteMsg, UserConfig};
    use cosmwasm_std::{
        attr, coin,
        testing::{mock_dependencies, mock_env, mock_info},
        Addr, BankMsg, DepsMut, MessageInfo, OverflowError, OverflowOperation, Response, Uint128,
    };

    use crate::{contract::execute, error::ContractError, state::USER_CONFIG};

    fn add_tip(deps: DepsMut, info: MessageInfo) {
        execute(deps, mock_env(), info, ExecuteMsg::AddBotTip {}).unwrap();
    }

    #[test]
    fn will_withdraw_tip() {
        let mut deps = mock_dependencies();

        let tip_sent = coin(10_000, "uusd");

        let info = mock_info("creator", &[]);
        let msg = ExecuteMsg::Withdraw {
            tip: tip_sent.amount,
        };

        // add bot tip
        add_tip(deps.as_mut(), mock_info("creator", &[tip_sent.clone()]));

        // attempt bot withdraw and check that we got the expected response
        let res = execute(deps.as_mut(), mock_env(), info, msg).unwrap();
        assert_eq!(
            res,
            Response::new()
                .add_attributes(vec![
                    attr("action", "withdraw"),
                    attr("tip_removed", tip_sent.amount)
                ])
                .add_message(BankMsg::Send {
                    to_address: "creator".to_string(),
                    amount: vec![tip_sent]
                })
        )
    }

    #[test]
    fn does_update_config() {
        let mut deps = mock_dependencies();

        let tip_sent = coin(10_000, "uusd");
        let tip_withdraw = coin(5_000, "uusd");

        let info = mock_info("creator", &[]);
        let msg = ExecuteMsg::Withdraw {
            tip: tip_withdraw.amount,
        };

        // add bot tip
        add_tip(deps.as_mut(), mock_info("creator", &[tip_sent]));

        // attempt bot withdraw and check that config was updated
        execute(deps.as_mut(), mock_env(), info, msg).unwrap();
        let config = USER_CONFIG
            .load(&deps.storage, &Addr::unchecked("creator"))
            .unwrap();
        assert_eq!(
            config,
            UserConfig {
                tip_balance: Uint128::from(5_000u64),
                ..UserConfig::default()
            }
        )
    }

    #[test]
    fn wont_excess_withdraw() {
        let mut deps = mock_dependencies();

        let tip_sent = coin(10_000, "uusd");
        let tip_withdraw = coin(15_000, "uusd");

        let info = mock_info("creator", &[]);
        let msg = ExecuteMsg::Withdraw {
            tip: tip_withdraw.amount,
        };

        // add bot tip
        add_tip(deps.as_mut(), mock_info("creator", &[tip_sent.clone()]));

        // attempt bot withdraw and check that it failed
        let err = execute(deps.as_mut(), mock_env(), info, msg).unwrap_err();
        assert_eq!(
            err,
            ContractError::OverflowError(OverflowError::new(
                OverflowOperation::Sub,
                tip_sent.amount,
                tip_withdraw.amount
            ))
        );
    }
}
