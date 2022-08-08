use astroport_dca::UserConfig;
use cosmwasm_std::{attr, Decimal, DepsMut, MessageInfo, Response};

use crate::{error::ContractError, helpers::ots, state::USER_CONFIG};

/// ## Description
/// Updates a users configuration with the specified parameters.
///
/// Returns a [`ContractError`] as a failure, otherwise returns a [`Response`] with the specified
/// attributes if the operation was successful.
/// ## Arguments
/// * `deps` - A [`DepsMut`] that contains the dependencies.
///
/// * `info` - A [`MessageInfo`] from the sender who wants to update their user configuration.
///
/// * `max_hops` - A `u8` value wrapped in an [`Option`] which represents the new maximum amount of
/// hops per DCA purchase. If `None`, the user will use the default config set by the contract.
///
/// * `max_spread` - A [`Decimal`] value wrapped in an [`Option`] which represents the new maximum
/// spread for each DCA purchase. If `None`, the user will use the config set by the contract.
pub fn update_user_config(
    deps: DepsMut,
    info: MessageInfo,
    max_hops: Option<u32>,
    max_spread: Option<Decimal>,
) -> Result<Response, ContractError> {
    let config = USER_CONFIG
        .may_load(deps.storage, &info.sender)?
        .unwrap_or_default();

    let new_config = UserConfig {
        max_hops,
        max_spread,
        ..config
    };

    USER_CONFIG.save(deps.storage, &info.sender, &new_config)?;

    Ok(Response::new().add_attributes(vec![
        attr("action", "update_user_config"),
        attr("max_hops", ots(&max_hops)),
        attr("max_spread", ots(&max_spread)),
    ]))
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use astroport::asset::{Asset, AssetInfo};
    use astroport_dca::{ExecuteMsg, UserConfig};
    use cosmwasm_std::{
        attr, coin,
        testing::{mock_dependencies, mock_env, mock_info},
        Decimal, Response, Uint128,
    };

    use crate::{
        contract::execute,
        state::{TIPS, USER_CONFIG},
    };

    #[test]
    fn does_update_user_config() {
        let mut deps = mock_dependencies();

        let info = mock_info("creator", &[]);
        let msg = ExecuteMsg::UpdateUserConfig {
            max_hops: Some(6),
            max_spread: Some(Decimal::from_str("0.025").unwrap()),
        };

        // does send the write response
        let res = execute(deps.as_mut(), mock_env(), info.clone(), msg).unwrap();
        assert_eq!(
            res,
            Response::new().add_attributes(vec![
                attr("action", "update_user_config"),
                attr("max_hops", "6"),
                attr("max_spread", "0.025")
            ])
        );

        // does update config
        let config = USER_CONFIG.load(&deps.storage, &info.sender).unwrap();
        assert_eq!(
            config,
            UserConfig {
                max_hops: Some(6),
                max_spread: Some(Decimal::from_str("0.025").unwrap()),
                tips_balance: vec![],
            }
        )
    }

    #[test]
    fn does_not_change_tip_balance() {
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

        let info = mock_info("creator", &[]);
        let msg = ExecuteMsg::UpdateUserConfig {
            max_hops: Some(6),
            max_spread: Some(Decimal::from_str("0.025").unwrap()),
        };

        // add tip
        let send_info = mock_info("creator", &[coin(10_000, "uusd")]);
        let send_tip_msg = ExecuteMsg::AddTips {};
        execute(deps.as_mut(), mock_env(), send_info.clone(), send_tip_msg).unwrap();

        // does not modify the tip balance
        execute(deps.as_mut(), mock_env(), info, msg).unwrap();

        let config = USER_CONFIG.load(&deps.storage, &send_info.sender).unwrap();
        assert_eq!(
            config.tips_balance,
            vec![Asset {
                info: AssetInfo::NativeToken {
                    denom: "uusd".to_string(),
                },
                amount: Uint128::new(10_000),
            }]
        );
    }

    #[test]
    fn does_reset_config() {
        let mut deps = mock_dependencies();

        let info = mock_info("creator", &[]);
        let update_msg = ExecuteMsg::UpdateUserConfig {
            max_hops: Some(4),
            max_spread: Some(Decimal::from_str("0.025").unwrap()),
        };
        let reset_msg = ExecuteMsg::UpdateUserConfig {
            max_hops: Some(6),
            max_spread: None,
        };

        // does reset the config
        execute(deps.as_mut(), mock_env(), info.clone(), update_msg).unwrap();
        execute(deps.as_mut(), mock_env(), info.clone(), reset_msg).unwrap();

        // does update config
        let config = USER_CONFIG.load(&deps.storage, &info.sender).unwrap();
        assert_eq!(
            config,
            UserConfig {
                max_hops: Some(6),
                max_spread: None,
                tips_balance: vec![],
            }
        )
    }
}
