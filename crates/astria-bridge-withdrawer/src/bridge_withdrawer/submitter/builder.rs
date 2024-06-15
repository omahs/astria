use std::sync::Arc;

use astria_eyre::eyre::{
    self,
    Context as _,
};
use tokio::sync::mpsc;
use tokio_util::sync::CancellationToken;
use tracing::info;

use super::state::State;
use crate::bridge_withdrawer::{
    startup,
    submitter::Batch,
};

const BATCH_QUEUE_SIZE: usize = 256;

pub(crate) struct Handle {
    batches_tx: mpsc::Sender<Batch>,
}

impl Handle {
    pub(crate) fn new(batches_tx: mpsc::Sender<Batch>) -> Self {
        Self {
            batches_tx,
        }
    }

    pub(crate) async fn send_batch(&self, batch: Batch) -> eyre::Result<()> {
        self.batches_tx
            .send(batch)
            .await
            .wrap_err("failed submitter_handleto send batch")
    }
}

pub(crate) struct Builder {
    pub(crate) shutdown_token: CancellationToken,
    pub(crate) startup_handle: startup::SubmitterHandle,
    pub(crate) sequencer_key_path: String,
    pub(crate) sequencer_cometbft_endpoint: String,
    pub(crate) state: Arc<State>,
}

impl Builder {
    /// Instantiates an `Submitter`.
    pub(crate) fn build(self) -> eyre::Result<(super::Submitter, Handle)> {
        let Self {
            shutdown_token,
            startup_handle,
            sequencer_key_path,
            sequencer_cometbft_endpoint,
            state,
        } = self;

        let signer = super::signer::SequencerKey::try_from_path(sequencer_key_path)
            .wrap_err("failed to load sequencer private ky")?;
        info!(address = %telemetry::display::hex(&signer.address), "loaded sequencer signer");

        let sequencer_cometbft_client =
            sequencer_client::HttpClient::new(&*sequencer_cometbft_endpoint)
                .wrap_err("failed constructing cometbft http client")?;

        let (batches_tx, batches_rx) = tokio::sync::mpsc::channel(BATCH_QUEUE_SIZE);
        let handle = Handle::new(batches_tx);

        Ok((
            super::Submitter {
                shutdown_token,
                startup_handle,
                state,
                batches_rx,
                sequencer_cometbft_client,
                signer,
            },
            handle,
        ))
    }
}
