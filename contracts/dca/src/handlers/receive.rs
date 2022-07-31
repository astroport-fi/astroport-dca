use astroport::asset::{addr_validate_to_lower, Asset, AssetInfo};
use astroport_dca::dca::ReceiveMsg;
use cosmwasm_std::{from_binary, DepsMut, Env, MessageInfo, Response};
use cw20::Cw20ReceiveMsg;

use crate::error::ContractError;

use super::add_bot_tip;

pub fn receive(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    cw20_msg: Cw20ReceiveMsg,
) -> Result<Response, ContractError> {
    let sender = addr_validate_to_lower(deps.api, cw20_msg.sender)?;
    match from_binary(&cw20_msg.msg)? {
        ReceiveMsg::AddBotTip {} => add_bot_tip(
            deps,
            sender,
            Asset {
                info: AssetInfo::Token {
                    contract_addr: info.sender,
                },
                amount: cw20_msg.amount,
            },
        ),
    }
}
