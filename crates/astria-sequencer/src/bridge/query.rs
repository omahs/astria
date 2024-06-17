use anyhow::Context as _;
use astria_core::{
    primitive::v1::Address,
    protocol::{
        abci::AbciErrorCode,
        bridge::v1alpha1::BridgeAccountInfo,
    },
};
use cnidarium::Storage;
use prost::Message as _;
use tendermint::abci::{
    request,
    response,
};

use crate::{
    bridge::state_ext::StateReadExt as _,
    state_ext::StateReadExt as _,
};

fn error_query_response(
    err: Option<anyhow::Error>,
    code: AbciErrorCode,
    info: &str,
) -> response::Query {
    if err.is_none() {
        return response::Query {
            code: code.into(),
            info: code.to_string(),
            log: info.into(),
            ..response::Query::default()
        };
    }

    let err = err.unwrap();
    response::Query {
        code: code.into(),
        info: code.to_string(),
        log: format!("{info}: {err:#}"),
        ..response::Query::default()
    }
}

pub(crate) async fn bridge_account_info_request(
    storage: Storage,
    request: request::Query,
    params: Vec<(String, String)>,
) -> response::Query {
    use astria_core::protocol::bridge::v1alpha1::BridgeAccountInfoResponse;

    let address = match preprocess_request(&params) {
        Ok(tup) => tup,
        Err(err_rsp) => return err_rsp,
    };

    let snapshot = storage.latest_snapshot();
    let height = match snapshot.get_block_height().await {
        Ok(height) => height,
        Err(err) => {
            return error_query_response(
                Some(err),
                AbciErrorCode::INTERNAL_ERROR,
                "failed to get block height",
            );
        }
    };

    let rollup_id = match snapshot.get_bridge_account_rollup_id(&address).await {
        Ok(Some(rollup_id)) => rollup_id,
        Ok(None) => {
            let resp = BridgeAccountInfoResponse {
                height,
                info: None,
            };
            let payload = resp.into_raw().encode_to_vec().into();

            let height =
                tendermint::block::Height::try_from(height).expect("height must fit into an i64");
            return response::Query {
                code: 0.into(),
                key: request.path.clone().into_bytes().into(),
                value: payload,
                height,
                ..response::Query::default()
            };
        }
        Err(err) => {
            return error_query_response(
                Some(err),
                AbciErrorCode::INTERNAL_ERROR,
                "failed to get rollup id",
            );
        }
    };

    let asset_id = match snapshot.get_bridge_account_asset_id(&address).await {
        Ok(asset_id) => asset_id,
        Err(err) => {
            return error_query_response(
                Some(err),
                AbciErrorCode::INTERNAL_ERROR,
                "failed to get asset id",
            );
        }
    };

    let sudo_address = match snapshot.get_bridge_account_sudo_address(&address).await {
        Ok(Some(sudo_address)) => sudo_address,
        Ok(None) => {
            return error_query_response(
                None,
                AbciErrorCode::INTERNAL_ERROR,
                "sudo address not set",
            );
        }
        Err(err) => {
            return error_query_response(
                Some(err),
                AbciErrorCode::INTERNAL_ERROR,
                "failed to get sudo address",
            );
        }
    };

    let withdrawer_address = match snapshot
        .get_bridge_account_withdrawer_address(&address)
        .await
    {
        Ok(Some(withdrawer_address)) => withdrawer_address,
        Ok(None) => {
            return error_query_response(
                None,
                AbciErrorCode::INTERNAL_ERROR,
                "withdrawer address not set",
            );
        }
        Err(err) => {
            return error_query_response(
                Some(err),
                AbciErrorCode::INTERNAL_ERROR,
                "failed to get withdrawer address",
            );
        }
    };

    let resp = BridgeAccountInfoResponse {
        height,
        info: Some(BridgeAccountInfo {
            rollup_id,
            asset_id,
            sudo_address,
            withdrawer_address,
        }),
    };

    let payload = resp.into_raw().encode_to_vec().into();

    let height = tendermint::block::Height::try_from(height).expect("height must fit into an i64");
    response::Query {
        code: 0.into(),
        key: request.path.clone().into_bytes().into(),
        value: payload,
        height,
        ..response::Query::default()
    }
}

pub(crate) async fn bridge_account_last_tx_hash_request(
    storage: Storage,
    request: request::Query,
    params: Vec<(String, String)>,
) -> response::Query {
    use astria_core::protocol::bridge::v1alpha1::BridgeAccountLastTxHashResponse;

    let address = match preprocess_request(&params) {
        Ok(tup) => tup,
        Err(err_rsp) => return err_rsp,
    };

    // use latest snapshot, as this is a query for latest tx
    let snapshot = storage.latest_snapshot();
    let height = match snapshot.get_block_height().await {
        Ok(height) => height,
        Err(err) => {
            return response::Query {
                code: AbciErrorCode::INTERNAL_ERROR.into(),
                info: AbciErrorCode::INTERNAL_ERROR.to_string(),
                log: format!("failed getting block height: {err:#}"),
                ..response::Query::default()
            };
        }
    };

    let resp = match snapshot
        .get_last_transaction_hash_for_bridge_account(&address)
        .await
    {
        Ok(Some(tx_hash)) => BridgeAccountLastTxHashResponse {
            height,
            tx_hash: Some(tx_hash),
        },
        Ok(None) => BridgeAccountLastTxHashResponse {
            height,
            tx_hash: None,
        },
        Err(err) => {
            return response::Query {
                code: AbciErrorCode::INTERNAL_ERROR.into(),
                info: AbciErrorCode::INTERNAL_ERROR.to_string(),
                log: format!("failed getting balance for provided address: {err:?}"),
                ..response::Query::default()
            };
        }
    };
    let payload = resp.into_raw().encode_to_vec().into();

    let height = tendermint::block::Height::try_from(height).expect("height must fit into an i64");
    response::Query {
        code: 0.into(),
        key: request.path.clone().into_bytes().into(),
        value: payload,
        height,
        ..response::Query::default()
    }
}

fn preprocess_request(params: &[(String, String)]) -> anyhow::Result<Address, response::Query> {
    let Some(address) = params
        .iter()
        .find_map(|(k, v)| (k == "address").then_some(v))
    else {
        return Err(response::Query {
            code: AbciErrorCode::INVALID_PARAMETER.into(),
            info: AbciErrorCode::INVALID_PARAMETER.to_string(),
            log: "path did not contain address parameter".into(),
            ..response::Query::default()
        });
    };
    let address = hex::decode(address)
        .context("failed decoding hex encoded bytes")
        .and_then(|addr| {
            crate::try_astria_address(&addr).context("failed constructing address from bytes")
        })
        .map_err(|err| response::Query {
            code: AbciErrorCode::INVALID_PARAMETER.into(),
            info: AbciErrorCode::INVALID_PARAMETER.to_string(),
            log: format!("address could not be constructed from provided parameter: {err:#}"),
            ..response::Query::default()
        })?;
    Ok(address)
}
