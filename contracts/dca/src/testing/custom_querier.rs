use std::collections::HashMap;

use cosmwasm_std::testing::BankQuerier;
use cosmwasm_std::{
    from_binary, from_slice, to_binary, Addr, Empty, Querier, QuerierResult, QueryRequest,
    SystemError, WasmQuery,
};
use cw20::Cw20QueryMsg;

use super::cw20_querier::Cw20Querier;
use super::helpers::err_unsupported_query;

#[derive(Default)]
pub(super) struct CustomQuerier {
    pub cw20_querier: Cw20Querier,
    pub bank_querier: BankQuerier,
}

impl Querier for CustomQuerier {
    fn raw_query(&self, bin_request: &[u8]) -> QuerierResult {
        let request: QueryRequest<_> = match from_slice(bin_request) {
            Ok(v) => v,
            Err(e) => {
                return Err(SystemError::InvalidRequest {
                    error: format!("Parsing query request: {}", e),
                    request: bin_request.into(),
                })
                .into()
            }
        };
        self.handle_query(&request)
    }
}

impl CustomQuerier {
    #[allow(dead_code)]
    pub fn set_cw20_balance(&mut self, token: &str, user: &str, balance: u128) {
        match self.cw20_querier.balances.get_mut(token) {
            Some(contract_balances) => {
                contract_balances.insert(user.to_string(), balance);
            }
            None => {
                let mut contract_balances: HashMap<String, u128> = HashMap::default();
                contract_balances.insert(user.to_string(), balance);
                self.cw20_querier
                    .balances
                    .insert(token.to_string(), contract_balances);
            }
        };
    }

    pub fn set_cw20_allowance(&mut self, token: &str, owner: &str, allowance: u128) {
        match self.cw20_querier.allowances.get_mut(token) {
            Some(allowances) => {
                allowances.insert(owner.to_string(), allowance);
            }
            None => {
                let mut allowances: HashMap<String, u128> = HashMap::default();
                allowances.insert(owner.to_string(), allowance);
                self.cw20_querier
                    .allowances
                    .insert(token.to_string(), allowances);
            }
        };
    }

    // pub fn set_cw20_total_supply(&mut self, token: &str, total_supply: u128) {
    //     self.cw20_querier
    //         .total_supplies
    //         .insert(token.to_string(), total_supply);
    // }

    // pub fn set_bank_balances(&mut self, balances: &[Coin]) {
    //     self.bank_querier = BankQuerier::new(&[(MOCK_CONTRACT_ADDR, balances)])
    // }

    pub fn handle_query(&self, request: &QueryRequest<Empty>) -> QuerierResult {
        match request {
            QueryRequest::Wasm(WasmQuery::Smart { contract_addr, msg }) => {
                if let Ok(query) = from_binary::<Cw20QueryMsg>(msg) {
                    return self.cw20_querier.handle_query(contract_addr, query);
                }

                if let Ok(query) = from_binary::<astroport::factory::QueryMsg>(msg) {
                    return match &query {
                        astroport::factory::QueryMsg::Config {} => {
                            Ok(to_binary(&astroport::factory::ConfigResponse {
                                owner: Addr::unchecked("factory_owner"),
                                pair_configs: vec![],
                                token_code_id: 1,
                                fee_address: None,
                                generator_address: None,
                                whitelist_code_id: 1,
                            })
                            .into())
                            .into()
                        }
                        _ => err_unsupported_query(msg),
                    };
                }

                err_unsupported_query(msg)
            }

            QueryRequest::Bank(query) => self.bank_querier.query(query),

            _ => err_unsupported_query(request),
        }
    }
}
