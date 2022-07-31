use astroport::asset::{Asset, AssetInfo};
use cosmwasm_std::{Addr, Decimal, StdError, StdResult, Storage};
use cw_storage_plus::{Index, IndexList, IndexedMap, Item, Map, MultiIndex};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use astroport_dca::dca::{DcaInfo, TipAssetInfo};

use crate::error::ContractError;

/// Stores the main dca module parameters.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Config {
    /// The maximum amount of hops to perform from `initial_asset` to `target_asset` when DCAing if the user does not specify
    pub max_hops: u32,
    /// The maximum amount of spread when performing a swap from `initial_asset` to `target_asset` when DCAing if the user does not specify
    pub max_spread: Decimal,
    /// The whitelisted tokens that can be used in a DCA purchase route
    pub whitelisted_tokens: Vec<AssetInfo>,
    /// The address of the Astroport factory contract
    pub factory_addr: Addr,
    /// The address of the Astroport router contract
    pub router_addr: Addr,
}

impl Config {
    pub fn is_whitelisted_asset(&self, asset: &AssetInfo) -> bool {
        self.whitelisted_tokens.contains(asset)
    }
}

pub(crate) struct State<'a> {
    /// The contract configuration
    pub config: Item<'a, Config>,

    /// Unique Id for dca_requests
    pub dca_id: Item<'a, u64>,

    /// DCA Strategies indexed by id and by user addr.
    pub dca_requests: IndexedMap<'a, u64, DcaInfo, DcaRequestsIndexes<'a>>,

    pub whitelisted_tip_tokens: Item<'a, Vec<TipAssetInfo>>,

    pub tip_jars: Map<'a, Addr, Vec<Asset>>,
    // Optionally it would be possible to use maps for the tip_tokens and tip_jars of a user
    // pub whitelisted_tip_tokens: Map<'a, String, Uint128>,
    // pub tip_jars: Map<'a, (Addr, String), Asset>,
}

impl Default for State<'static> {
    fn default() -> Self {
        let dca_indexes = DcaRequestsIndexes {
            user: MultiIndex::new(
                |d: &DcaInfo| d.user.clone().into(),
                "dca_requests",
                "dca_requests__user",
            ),
            user_asset: MultiIndex::new(
                |d: &DcaInfo| (d.user.clone().into(), d.initial_asset.info.to_string()),
                "dca_requests",
                "dca_requests__asset",
            ),
        };

        Self {
            config: Item::new("config"),
            dca_id: Item::new("dca_id"),
            dca_requests: IndexedMap::new("dca_requests", dca_indexes),

            whitelisted_tip_tokens: Item::new("whitelisted_tip_tokens"),
            tip_jars: Map::new("tip_jars"),
        }
    }
}

impl State<'static> {
    pub fn assert_whitelisted_tip_asset(
        &self,
        storage: &dyn Storage,
        asset: AssetInfo,
    ) -> Result<(), ContractError> {
        let whitelisted_tip_tokens = self.whitelisted_tip_tokens.load(storage)?;

        if whitelisted_tip_tokens
            .iter()
            .any(|token| token.info == asset)
        {
            Ok(())
        } else {
            Err(ContractError::InvalidBotTipToken {
                token: asset.to_string(),
            })
        }
    }

    pub fn get_next_order_id(&self, storage: &mut dyn Storage) -> Result<u64, ContractError> {
        let next_order_id =
            self.dca_id
                .load(storage)?
                .checked_add(1u64)
                .ok_or(StdError::generic_err(
                    "could not calculate order_id".to_string(),
                ))?;

        self.dca_id.save(storage, &next_order_id)?;
        Ok(next_order_id)
    }

    pub fn get_tip_jars(&self, storage: &dyn Storage, addr: Addr) -> StdResult<Vec<Asset>> {
        self.tip_jars.load(storage, addr)

        // self.tip_jars
        //     .prefix(addr)
        //     .range(storage, None, None, Order::Ascending)
        //     .map(|item| {
        //         let (_, v) = item?;
        //         Ok(v)
        //     })
        //     .collect()
    }
}

pub(crate) struct DcaRequestsIndexes<'a> {
    pub user: MultiIndex<'a, String, DcaInfo, u64>,

    pub user_asset: MultiIndex<'a, (String, String), DcaInfo, u64>,
}

impl<'a> IndexList<DcaInfo> for DcaRequestsIndexes<'a> {
    fn get_indexes(&'_ self) -> Box<dyn Iterator<Item = &'_ dyn Index<DcaInfo>> + '_> {
        let v: Vec<&dyn Index<DcaInfo>> = vec![&self.user, &self.user_asset];
        Box::new(v.into_iter())
    }
}
