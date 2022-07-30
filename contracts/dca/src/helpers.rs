use astroport::asset::AssetInfo;
use cosmwasm_std::{to_binary, Addr, BankMsg, Coin, CosmosMsg, StdResult, Uint128, WasmMsg};
use cw20::Cw20ExecuteMsg;

pub fn asset_transfer(info: &AssetInfo, amount: Uint128, to: &Addr) -> StdResult<CosmosMsg> {
    Ok(match &info {
        AssetInfo::Token { contract_addr } => CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: contract_addr.to_string(),
            msg: to_binary(&Cw20ExecuteMsg::Transfer {
                recipient: to.to_string(),
                amount,
            })?,
            funds: vec![],
        }),
        AssetInfo::NativeToken { denom } => CosmosMsg::Bank(BankMsg::Send {
            to_address: to.to_string(),
            amount: vec![Coin {
                denom: denom.to_string(),
                amount,
            }],
        }),
    })
}
