use astroport::asset::Asset;
use cosmwasm_std::{attr, DepsMut, MessageInfo, Response};

use crate::{
    error::ContractError,
    state::{TIPS, USER_CONFIG},
};

/// ## Description
/// Adds a tip to the contract for a users DCA purchases.
///
/// Returns a [`ContractError`] as a failure, otherwise returns a [`Response`] with the specified
/// attributes if the operation was successful.
/// ## Arguments
/// * `deps` - A [`DepsMut`] that contains the dependencies.
///
/// * `info` - A [`MessageInfo`] which contains a uusd tip to add to a users tip balance.
pub fn add_bot_tip(
    deps: DepsMut,
    info: MessageInfo,
    tips: Vec<Asset>,
) -> Result<Response, ContractError> {
    (!tips.is_empty())
        .then(|| ())
        .ok_or(ContractError::InvalidZeroAmount {})?;

    let tips_denom = TIPS
        .load(deps.storage)?
        .into_iter()
        .map(|e| e.info)
        .collect::<Vec<_>>();

    // update user tip in contract
    USER_CONFIG.update(
        deps.storage,
        &info.sender,
        |config| -> Result<_, ContractError> {
            let mut config = config.unwrap_or_default();

            for tip in tips {
                match tips_denom.contains(&tip.info) {
                    true => match config.tips_balance.iter_mut().find(|e| e.info == tip.info) {
                        Some(balance) => {
                            balance.amount += tip.amount;
                        }
                        None => config.tips_balance.push(tip),
                    },
                    false => Err(ContractError::InvalidTipAssetInfo {})?,
                };
            }

            Ok(config)
        },
    )?;

    Ok(Response::new().add_attributes(vec![attr("action", "add_bot_tip")]))
}

#[cfg(test)]
mod tests {
    use astroport::asset::{Asset, AssetInfo};
    use astroport_dca::{ExecuteMsg, UserConfig};
    use cosmwasm_std::{
        attr, coin,
        testing::{mock_dependencies, mock_env, mock_info},
        Addr, Response, Uint128,
    };

    use crate::{
        contract::execute,
        error::ContractError,
        state::{TIPS, USER_CONFIG},
    };

    #[test]
    fn does_add_bot_tip() {
        let mut deps = mock_dependencies();

        let tip_sent = coin(10000, "uusd");

        TIPS.save(
            &mut deps.storage,
            &vec![Asset {
                info: AssetInfo::NativeToken {
                    denom: "uusd".to_string(),
                },
                amount: Uint128::new(100),
            }],
        )
        .unwrap();

        let info = mock_info("creator", &[tip_sent.clone()]);
        let msg = ExecuteMsg::AddBotTip {};

        // check that we got the expected response
        let res = execute(deps.as_mut(), mock_env(), info, msg).unwrap();
        assert_eq!(
            res,
            Response::new().add_attributes(vec![attr("action", "add_bot_tip"),])
        );

        // check that user tip balance was added
        let config = USER_CONFIG
            .load(&deps.storage, &Addr::unchecked("creator"))
            .unwrap();
        assert_eq!(
            config,
            UserConfig {
                tips_balance: vec![Asset {
                    info: AssetInfo::NativeToken {
                        denom: "uusd".to_string()
                    },
                    amount: tip_sent.amount
                }],
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

        TIPS.save(
            &mut deps.storage,
            &vec![Asset {
                info: AssetInfo::NativeToken {
                    denom: "uusd".to_string(),
                },
                amount: Uint128::new(100),
            }],
        )
        .unwrap();

        let info = mock_info("creator", &[coin(20000, "ukrw")]);
        let msg = ExecuteMsg::AddBotTip {};

        // should error with InvalidZeroAmount
        let res = execute(deps.as_mut(), mock_env(), info, msg).unwrap_err();
        assert_eq!(res, ContractError::InvalidTipAssetInfo {});
    }
}
