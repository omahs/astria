/// A response containing the last tx hash given some bridge address,
/// if it exists.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct BridgeAccountLastTxHashResponse {
    #[prost(uint64, tag = "2")]
    pub height: u64,
    #[prost(bytes = "vec", optional, tag = "3")]
    pub tx_hash: ::core::option::Option<::prost::alloc::vec::Vec<u8>>,
}
impl ::prost::Name for BridgeAccountLastTxHashResponse {
    const NAME: &'static str = "BridgeAccountLastTxHashResponse";
    const PACKAGE: &'static str = "astria.protocol.bridge.v1alpha1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("astria.protocol.bridge.v1alpha1.{}", Self::NAME)
    }
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct BridgeAccountInfoResponse {
    #[prost(uint64, tag = "2")]
    pub height: u64,
    #[prost(message, optional, tag = "3")]
    pub rollup_id: ::core::option::Option<super::super::super::primitive::v1::RollupId>,
    #[prost(bytes = "vec", optional, tag = "4")]
    pub asset_id: ::core::option::Option<::prost::alloc::vec::Vec<u8>>,
    #[prost(message, optional, tag = "5")]
    pub sudo_address: ::core::option::Option<
        super::super::super::primitive::v1::Address,
    >,
    #[prost(message, optional, tag = "6")]
    pub withdrawer_address: ::core::option::Option<
        super::super::super::primitive::v1::Address,
    >,
}
impl ::prost::Name for BridgeAccountInfoResponse {
    const NAME: &'static str = "BridgeAccountInfoResponse";
    const PACKAGE: &'static str = "astria.protocol.bridge.v1alpha1";
    fn full_name() -> ::prost::alloc::string::String {
        ::prost::alloc::format!("astria.protocol.bridge.v1alpha1.{}", Self::NAME)
    }
}
