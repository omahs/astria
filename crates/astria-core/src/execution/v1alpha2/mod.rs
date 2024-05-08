use bytes::Bytes;
use pbjson_types::Timestamp;

use crate::{
    generated::execution::v1alpha2 as raw,
    primitive::v1::{
        IncorrectRollupIdLength,
        RollupId,
    },
    Protobuf,
};

// An error when transforming a [`raw::GenesisInfo`] into a [`GenesisInfo`].
#[derive(Debug, thiserror::Error)]
#[error(transparent)]
pub struct GenesisInfoError(GenesisInfoErrorKind);

impl GenesisInfoError {
    fn incorrect_rollup_id_length(inner: IncorrectRollupIdLength) -> Self {
        Self(GenesisInfoErrorKind::IncorrectRollupIdLength(inner))
    }
}

#[derive(Debug, thiserror::Error)]
enum GenesisInfoErrorKind {
    #[error("`rollup_id` field did not contain a valid rollup ID")]
    IncorrectRollupIdLength(IncorrectRollupIdLength),
}

/// Genesis Info required from a rollup to start a an execution client.
///
/// Contains information about the rollup id, and base heights for both sequencer & celestia.
///
/// Usually constructed its [`Protobuf`] implementation from a
/// [`raw::GenesisInfo`].
#[derive(Clone, Copy, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
#[cfg_attr(
    feature = "serde",
    serde(into = "crate::generated::execution::v1alpha2::GenesisInfo")
)]
pub struct GenesisInfo {
    /// The rollup id which is used to identify the rollup txs.
    rollup_id: RollupId,
    /// The allowed variance in the block height of celestia when looking for sequencer blocks.
    celestia_block_variance: u32,
}

impl GenesisInfo {
    #[must_use]
    pub fn rollup_id(&self) -> RollupId {
        self.rollup_id
    }

    #[must_use]
    pub fn celestia_block_variance(&self) -> u32 {
        self.celestia_block_variance
    }
}

impl From<GenesisInfo> for raw::GenesisInfo {
    fn from(value: GenesisInfo) -> Self {
        value.to_raw()
    }
}

impl Protobuf for GenesisInfo {
    type Error = GenesisInfoError;
    type Raw = raw::GenesisInfo;

    fn try_from_raw_ref(raw: &Self::Raw) -> Result<Self, Self::Error> {
        let raw::GenesisInfo {
            rollup_id,
            celestia_block_variance,
        } = raw;
        let rollup_id =
            RollupId::try_from_slice(rollup_id).map_err(Self::Error::incorrect_rollup_id_length)?;

        Ok(Self {
            rollup_id,
            celestia_block_variance: *celestia_block_variance,
        })
    }

    fn to_raw(&self) -> Self::Raw {
        let Self {
            rollup_id,
            celestia_block_variance,
        } = self;
        Self::Raw {
            rollup_id: Bytes::copy_from_slice(rollup_id.as_ref()),
            celestia_block_variance: *celestia_block_variance,
        }
    }
}

/// An error when transforming a [`raw::Block`] into a [`Block`].
#[derive(Debug, thiserror::Error)]
#[error(transparent)]
pub struct BlockError(BlockErrorKind);

impl BlockError {
    fn field_not_set(field: &'static str) -> Self {
        Self(BlockErrorKind::FieldNotSet(field))
    }
}

#[derive(Debug, thiserror::Error)]
enum BlockErrorKind {
    #[error("{0} field not set")]
    FieldNotSet(&'static str),
}

/// An Astria execution block on a rollup.
///
/// Contains information about the block number, its hash,
/// its parent block's hash, and timestamp.
///
/// Usually constructed its [`Protobuf`] implementation from a
/// [`raw::Block`].
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
#[cfg_attr(
    feature = "serde",
    serde(into = "crate::generated::execution::v1alpha2::Block")
)]
pub struct Block {
    /// The block number
    number: u32,
    /// The hash of the block
    hash: Bytes,
    /// The hash of the parent block
    parent_block_hash: Bytes,
    /// Timestamp on the block, standardized to google protobuf standard.
    timestamp: Timestamp,
}

impl Block {
    #[must_use]
    pub fn number(&self) -> u32 {
        self.number
    }

    #[must_use]
    pub fn hash(&self) -> &Bytes {
        &self.hash
    }

    #[must_use]
    pub fn parent_block_hash(&self) -> &Bytes {
        &self.parent_block_hash
    }

    #[must_use]
    pub fn timestamp(&self) -> Timestamp {
        // prost_types::Timestamp is a (i64, i32) tuple, so this is
        // effectively just a copy
        self.timestamp.clone()
    }
}

impl From<Block> for raw::Block {
    fn from(value: Block) -> Self {
        value.to_raw()
    }
}

impl Protobuf for Block {
    type Error = BlockError;
    type Raw = raw::Block;

    fn try_from_raw_ref(raw: &Self::Raw) -> Result<Self, Self::Error> {
        let raw::Block {
            number,
            hash,
            parent_block_hash,
            timestamp,
        } = raw;
        // Cloning timestamp is effectively a copy because timestamp is just a (i32, i64) tuple
        let timestamp = timestamp
            .clone()
            .ok_or(Self::Error::field_not_set(".timestamp"))?;

        Ok(Self {
            number: *number,
            hash: hash.clone(),
            parent_block_hash: parent_block_hash.clone(),
            timestamp,
        })
    }

    fn to_raw(&self) -> Self::Raw {
        let Self {
            number,
            hash,
            parent_block_hash,
            timestamp,
        } = self;
        Self::Raw {
            number: *number,
            hash: hash.clone(),
            parent_block_hash: parent_block_hash.clone(),
            // Cloning timestamp is effectively a copy because timestamp is just a (i32, i64)
            // tuple
            timestamp: Some(timestamp.clone()),
        }
    }
}

#[derive(Debug, thiserror::Error)]
#[error(transparent)]
pub struct CommitmentStateError(CommitmentStateErrorKind);

impl CommitmentStateError {
    fn field_not_set(field: &'static str) -> Self {
        Self(CommitmentStateErrorKind::FieldNotSet(field))
    }

    fn firm(source: BlockError) -> Self {
        Self(CommitmentStateErrorKind::Firm(source))
    }

    fn soft(source: BlockError) -> Self {
        Self(CommitmentStateErrorKind::Soft(source))
    }

    fn firm_exceeds_soft(source: FirmExceedsSoft) -> Self {
        Self(CommitmentStateErrorKind::FirmExceedsSoft(source))
    }
}

#[derive(Debug, thiserror::Error)]
enum CommitmentStateErrorKind {
    #[error("{0} field not set")]
    FieldNotSet(&'static str),
    #[error(".firm field did not contain a valid block")]
    Firm(#[source] BlockError),
    #[error(".soft field did not contain a valid block")]
    Soft(#[source] BlockError),
    #[error(transparent)]
    FirmExceedsSoft(FirmExceedsSoft),
}

#[derive(Debug, thiserror::Error)]
#[error("firm commitment at `{firm} exceeds soft commitment at `{soft}")]
pub struct FirmExceedsSoft {
    firm: u32,
    soft: u32,
}

pub struct NoFirm;
pub struct NoSoft;
pub struct NoNextSequencerHeight;
pub struct NoBaseCelestiaHeight;
pub struct WithFirm(Block);
pub struct WithSoft(Block);
pub struct WithNextSequencerHeight(u32);
pub struct WithBaseCelestiaHeight(u32);

#[derive(Default)]
pub struct CommitmentStateBuilder<TFirm = NoFirm, TSoft = NoSoft, TNextSequencerHeight = NoNextSequencerHeight, TBaseCelestiaHeight = NoBaseCelestiaHeight> {
    firm: TFirm,
    soft: TSoft,
    next_sequencer_height: TNextSequencerHeight,
    base_celestia_height: TBaseCelestiaHeight,
}

impl CommitmentStateBuilder<NoFirm, NoSoft, NoNextSequencerHeight, NoBaseCelestiaHeight> {
    fn new() -> Self {
        Self {
            firm: NoFirm,
            soft: NoSoft,
            next_sequencer_height: NoNextSequencerHeight,
            base_celestia_height: NoBaseCelestiaHeight,
        }
    }
}

impl<TFirm, TSoft, TNextSequencerHeight, TBaseCelestiaHeight> CommitmentStateBuilder<TFirm, TSoft, TNextSequencerHeight, TBaseCelestiaHeight> {
    pub fn firm(self, firm: Block) -> CommitmentStateBuilder<WithFirm, TSoft, TNextSequencerHeight, TBaseCelestiaHeight> {
        let Self {
            soft,
            next_sequencer_height,
            base_celestia_height,..
        } = self;
        CommitmentStateBuilder {
            firm: WithFirm(firm),
            soft,
            next_sequencer_height,
            base_celestia_height,
        }
    }

    pub fn soft(self, soft: Block) -> CommitmentStateBuilder<TFirm, WithSoft, TNextSequencerHeight, TBaseCelestiaHeight> {
        let Self {
            firm,
            next_sequencer_height,
            base_celestia_height,..
        } = self;
        CommitmentStateBuilder {
            firm,
            soft: WithSoft(soft),
            next_sequencer_height,
            base_celestia_height,
        }
    }

    pub fn next_sequencer_height(self, next_sequencer_height: u32) -> CommitmentStateBuilder<TFirm, TSoft, WithNextSequencerHeight, TBaseCelestiaHeight> {
        let Self {
            firm,
            soft,
            base_celestia_height,..
        } = self;
        CommitmentStateBuilder {
            firm,
            soft,
            next_sequencer_height: WithNextSequencerHeight(next_sequencer_height),
            base_celestia_height,
        }
    }

    pub fn base_celestia_height(self, base_celestia_height: u32) -> CommitmentStateBuilder<TFirm, TSoft, TNextSequencerHeight, WithBaseCelestiaHeight> {
        let Self {
            firm,
            soft,
            next_sequencer_height,..
        } = self;
        CommitmentStateBuilder {
            firm,
            soft,
            next_sequencer_height,
            base_celestia_height: WithBaseCelestiaHeight(base_celestia_height)
        }
    }
    
    
}

impl CommitmentStateBuilder<WithFirm, WithSoft, WithNextSequencerHeight, WithBaseCelestiaHeight> {
    /// Finalize the commitment state.
    ///
    /// # Errors
    /// Returns an error if the firm block exceeds the soft one.
    pub fn build(self) -> Result<CommitmentState, FirmExceedsSoft> {
        let Self {
            firm: WithFirm(firm),
            soft: WithSoft(soft),
            next_sequencer_height: WithNextSequencerHeight(next_sequencer_height),
            base_celestia_height: WithBaseCelestiaHeight(base_celestia_height),
        } = self;
        if firm.number() > soft.number() {
            return Err(FirmExceedsSoft {
                firm: firm.number(),
                soft: soft.number(),
            });
        }
        Ok(CommitmentState {
            soft,
            firm,
            next_sequencer_height,
            base_celestia_height,
        })
    }
}

/// Information about the [`Block`] at each sequencer commitment level.
///
/// A commitment state is valid if:
/// - Block numbers are such that soft >= firm (upheld by this type).
/// - No blocks ever decrease in block number.
/// - The chain defined by soft is the head of the canonical chain the firm block must belong to.
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
#[cfg_attr(
    feature = "serde",
    serde(into = "crate::generated::execution::v1alpha2::CommitmentState")
)]
pub struct CommitmentState {
    /// Soft commitment is the rollup block matching latest sequencer block.
    soft: Block,
    /// Firm commitment is achieved when data has been seen in DA.
    firm: Block,
    
    next_sequencer_height: u32,
    
    base_celestia_height: u32,
}

impl CommitmentState {
    #[must_use = "a commitment state must be built to be useful"]
    pub fn builder() -> CommitmentStateBuilder {
        CommitmentStateBuilder::new()
    }

    #[must_use]
    pub fn firm(&self) -> &Block {
        &self.firm
    }

    #[must_use]
    pub fn soft(&self) -> &Block {
        &self.soft
    }

    #[must_use]
    pub fn next_sequencer_height(&self) -> u32 {
        self.next_sequencer_height
    }

    #[must_use]
    pub fn base_celestia_height(&self) -> u32 {
        self.base_celestia_height
    }
}

impl From<CommitmentState> for raw::CommitmentState {
    fn from(value: CommitmentState) -> Self {
        value.to_raw()
    }
}

impl Protobuf for CommitmentState {
    type Error = CommitmentStateError;
    type Raw = raw::CommitmentState;

    fn try_from_raw_ref(raw: &Self::Raw) -> Result<Self, Self::Error> {
        let Self::Raw {
            soft,
            firm,
            next_sequencer_height,
            base_celestia_height,
        } = raw;
        let soft = 'soft: {
            let Some(soft) = soft else {
                break 'soft Err(Self::Error::field_not_set(".soft"));
            };
            Block::try_from_raw_ref(soft).map_err(Self::Error::soft)
        }?;
        let firm = 'firm: {
            let Some(firm) = firm else {
                break 'firm Err(Self::Error::field_not_set(".firm"));
            };
            Block::try_from_raw_ref(firm).map_err(Self::Error::firm)
        }?;
        Self::builder()
            .firm(firm)
            .soft(soft)
            .next_sequencer_height(*next_sequencer_height)
            .base_celestia_height(*base_celestia_height)
            .build()
            .map_err(Self::Error::firm_exceeds_soft)
    }

    fn to_raw(&self) -> Self::Raw {
        let Self {
            soft,
            firm,
            next_sequencer_height,
            base_celestia_height,
        } = self;
        let soft = soft.to_raw();
        let firm = firm.to_raw();
        Self::Raw {
            soft: Some(soft),
            firm: Some(firm),
            next_sequencer_height: *next_sequencer_height,
            base_celestia_height: *base_celestia_height,
        }
    }
}
