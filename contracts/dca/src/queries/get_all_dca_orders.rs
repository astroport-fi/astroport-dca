use astroport_dca::DcaInfo;
use cosmwasm_std::{Deps, Order, StdResult};
use cw_storage_plus::Bound;

use crate::state::DCA;

const ORDER_LIMIT: u64 = 50;

/// ## Description
/// Returns all DCA orders currently set.
///
/// The result is returned in a [`Vec<DcaInfo>`] object.
pub fn get_all_dca_orders(
    deps: Deps,
    start_after: Option<u64>,
    limit: Option<u64>,
    is_ascending: Option<bool>,
) -> StdResult<Vec<DcaInfo>> {
    let bound = match is_ascending.unwrap_or(false) {
        true => (start_after.map(Bound::exclusive), None, Order::Ascending),
        false => (None, start_after.map(Bound::exclusive), Order::Descending),
    };

    DCA.range(deps.storage, bound.0, bound.1, bound.2)
        .map(|e| -> StdResult<_> { Ok(e?.1) })
        .take(limit.unwrap_or(ORDER_LIMIT) as usize)
        .collect::<StdResult<Vec<_>>>()
}
