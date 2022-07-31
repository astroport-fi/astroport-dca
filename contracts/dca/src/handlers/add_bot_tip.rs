use astroport::asset::Asset;
use cosmwasm_std::{attr, Addr, DepsMut, Response, StdResult};

use crate::{error::ContractError, state::State};

/// ## Description
/// Adds a tip to the contract for a users DCA purchases.
///
/// Returns a [`ContractError`] as a failure, otherwise returns a [`Response`] with the specified
/// attributes if the operation was successful.
/// ## Arguments
/// * `deps` - A [`DepsMut`] that contains the dependencies.
///
/// * `info` - A [`MessageInfo`] which contains a uusd tip to add to a users tip balance.
pub fn add_bot_tip(deps: DepsMut, sender: Addr, asset: Asset) -> Result<Response, ContractError> {
    let state = State::default();

    if asset.amount.is_zero() {
        return Err(ContractError::InvalidZeroAmount {});
    }

    state.assert_whitelisted_tip_asset(deps.storage, asset.info.clone())?;

    state
        .tip_jars
        .update(deps.storage, sender, |tip_jars| -> StdResult<Vec<Asset>> {
            let mut tip_jars = tip_jars.unwrap_or_else(|| vec![]);

            match tip_jars.iter_mut().find(|jar| jar.info == asset.info) {
                Some(tip_jar) => {
                    tip_jar.amount = tip_jar.amount.checked_add(asset.amount)?;
                }
                None => {
                    tip_jars.push(asset.clone());
                }
            }

            Ok(tip_jars)
        })?;

    Ok(Response::new().add_attributes(vec![
        attr("action", "add_bot_tip"),
        attr("tip_amount", asset.amount),
    ]))
}

#[cfg(test)]
mod tests {
    use astroport::asset::{Asset, AssetInfo};
    use astroport_dca::dca::{ExecuteMsg, TipAssetInfo};
    use cosmwasm_std::{
        attr, coin,
        testing::{mock_dependencies, mock_env, mock_info},
        Addr, Response, Uint128,
    };

    use crate::{contract::execute, error::ContractError, state::State};

    #[test]
    fn does_add_bot_tip_native() {
        let mut deps = mock_dependencies();
        let state = State::default();

        state
            .whitelisted_tip_tokens
            .save(
                deps.as_mut().storage,
                &vec![TipAssetInfo {
                    info: AssetInfo::NativeToken {
                        denom: "uluna".to_string(),
                    },
                    per_hop_fee: Uint128::new(100),
                }],
            )
            .unwrap();

        let tip_sent = coin(10000, "uluna");

        let info = mock_info("creator", &[tip_sent.clone()]);
        let msg = ExecuteMsg::AddBotTip {
            asset: Asset {
                info: AssetInfo::NativeToken {
                    denom: "uluna".to_string(),
                },
                amount: tip_sent.amount,
            },
        };

        // check that we got the expected response
        let res = execute(deps.as_mut(), mock_env(), info.clone(), msg).unwrap();
        assert_eq!(
            res,
            Response::new().add_attributes(vec![
                attr("action", "add_bot_tip"),
                attr("tip_amount", tip_sent.amount)
            ])
        );

        let jars = state.get_tip_jars(&deps.storage, info.sender).unwrap();

        assert_eq!(
            jars,
            vec![Asset {
                amount: tip_sent.amount,
                info: AssetInfo::NativeToken {
                    denom: "uluna".to_string()
                }
            }]
        )
    }

    #[test]
    fn does_add_bot_tip_cw20() {
        let mut deps = mock_dependencies();
        let state = State::default();

        state
            .whitelisted_tip_tokens
            .save(
                deps.as_mut().storage,
                &vec![TipAssetInfo {
                    info: AssetInfo::Token {
                        contract_addr: Addr::unchecked("token"),
                    },
                    per_hop_fee: Uint128::new(100),
                }],
            )
            .unwrap();

        let tip_sent = coin(10000, "uluna");

        //
        // TEST for unsupported NATIVE
        //
        let info = mock_info("creator", &[tip_sent.clone()]);
        let msg = ExecuteMsg::AddBotTip {
            asset: Asset {
                info: AssetInfo::NativeToken {
                    denom: "uluna".to_string(),
                },
                amount: tip_sent.amount,
            },
        };

        // check that we got the expected response
        let res = execute(deps.as_mut(), mock_env(), info.clone(), msg).unwrap_err();
        assert_eq!(
            res,
            ContractError::InvalidBotTipToken {
                token: "uluna".to_string()
            }
        );

        //
        // TEST for unsupported CW20
        //
        let info = mock_info("creator", &[]);
        let msg = ExecuteMsg::AddBotTip {
            asset: Asset {
                info: AssetInfo::Token {
                    contract_addr: Addr::unchecked("unknown"),
                },
                amount: tip_sent.amount,
            },
        };

        // check that we got the expected response
        let res = execute(deps.as_mut(), mock_env(), info.clone(), msg).unwrap_err();
        assert_eq!(
            res,
            ContractError::InvalidBotTipToken {
                token: "unknown".to_string()
            }
        );

        //
        // TEST for supported CW20
        //
        let info = mock_info("creator", &[]);
        let msg = ExecuteMsg::AddBotTip {
            asset: Asset {
                info: AssetInfo::Token {
                    contract_addr: Addr::unchecked("token".to_string()),
                },
                amount: tip_sent.amount,
            },
        };

        // check that we got the expected response
        let res = execute(deps.as_mut(), mock_env(), info.clone(), msg).unwrap();
        assert_eq!(
            res,
            Response::new().add_attributes(vec![
                attr("action", "add_bot_tip"),
                attr("tip_amount", tip_sent.amount)
            ])
        );

        let jars = state.get_tip_jars(&deps.storage, info.sender).unwrap();

        assert_eq!(
            jars,
            vec![Asset {
                amount: tip_sent.amount,
                info: AssetInfo::Token {
                    contract_addr: Addr::unchecked("token".to_string())
                }
            }]
        )
    }

    #[test]
    fn does_require_whitelisted() {
        let mut deps = mock_dependencies();

        let state = State::default();
        state
            .whitelisted_tip_tokens
            .save(
                deps.as_mut().storage,
                &vec![TipAssetInfo {
                    info: AssetInfo::NativeToken {
                        denom: "uluna".to_string(),
                    },
                    per_hop_fee: Uint128::new(100),
                }],
            )
            .unwrap();

        let info = mock_info("creator", &[]);
        let msg = ExecuteMsg::AddBotTip {
            asset: Asset {
                info: AssetInfo::NativeToken {
                    denom: "uluna".to_string(),
                },
                amount: Uint128::zero(),
            },
        };

        // should error with InvalidZeroAmount failure
        let res = execute(deps.as_mut(), mock_env(), info, msg).unwrap_err();
        assert_eq!(res, ContractError::InvalidZeroAmount {});
    }
}
