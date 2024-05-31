// @generated
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct BridgeState {
    #[prost(uint64, tag="1")]
    pub id: u64,
    #[prost(string, tag="2")]
    pub name: ::prost::alloc::string::String,
    #[prost(bool, tag="3")]
    pub is_enabled: bool,
}
/// each bridge have multiple connections to different chains
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Connection {
    /// = bridgeName/chainId
    #[prost(string, tag="1")]
    pub connection_id: ::prost::alloc::string::String,
    #[prost(uint64, tag="2")]
    pub bridge_id: u64,
    #[prost(string, tag="3")]
    pub chain_id: ::prost::alloc::string::String,
    #[prost(string, tag="4")]
    pub chain_display_name: ::prost::alloc::string::String,
    #[prost(string, tag="5")]
    pub gateway_address: ::prost::alloc::string::String,
    #[prost(string, tag="6")]
    pub encoding: ::prost::alloc::string::String,
    #[prost(string, tag="7")]
    pub escrow_address: ::prost::alloc::string::String,
    #[prost(bool, tag="8")]
    pub is_enabled: bool,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ControllerContracts {
    #[prost(string, tag="1")]
    pub id: ::prost::alloc::string::String,
    #[prost(string, tag="2")]
    pub token_controller: ::prost::alloc::string::String,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ControllersToUpdate {
    #[prost(message, optional, tag="1")]
    pub token_controller: ::core::option::Option<::pbjson_types::StringValue>,
}
/// each connection can have multiple external tokens, which contains the mapping
/// to native denom
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ExternalTokenMapping {
    #[prost(string, tag="1")]
    pub connection_id: ::prost::alloc::string::String,
    #[prost(bool, tag="2")]
    pub is_enabled: bool,
    #[prost(bool, tag="3")]
    pub is_carbon_owned: bool,
    #[prost(string, tag="4")]
    pub external_address: ::prost::alloc::string::String,
    /// corresponding carbon native denom
    #[prost(string, tag="5")]
    pub denom: ::prost::alloc::string::String,
}
/// contracts from external chains that can be executed by carbon
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ExecutableContract {
    #[prost(string, tag="1")]
    pub connection_id: ::prost::alloc::string::String,
    #[prost(string, tag="2")]
    pub address: ::prost::alloc::string::String,
}
/// RelayDetails defines the details of the relay
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct RelayDetails {
    #[prost(string, tag="1")]
    pub fee_receiver_address: ::prost::alloc::string::String,
    #[prost(string, tag="2")]
    pub fee_sender_address: ::prost::alloc::string::String,
    #[prost(message, optional, tag="3")]
    pub fee: ::core::option::Option<::cosmos_sdk_proto::cosmos::base::v1beta1::Coin>,
    #[prost(int64, tag="4")]
    pub block_created_at: i64,
    #[prost(message, optional, tag="5")]
    pub block_sent_at: ::core::option::Option<::pbjson_types::Int64Value>,
}
/// Params defines the parameters for the module.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Params {
    /// axelar_ibc_channel the IBC channel that currently connects to axelar
    /// blockchain
    #[prost(string, tag="1")]
    pub axelar_ibc_channel: ::prost::alloc::string::String,
    /// ibc_timeout_height_offset specifies the number of blocks to be added to the
    /// current block height of the destination chain to determine the timeout
    /// height for IBC messages. This offset is used to calculate the block height
    /// on the destination chain at which the message will timeout if not
    /// processed. For example, an offset of 200 means that the message will
    /// timeout if it is not relayed and processed within 200 blocks from the
    /// current height of the destination chain.
    #[prost(uint64, tag="2")]
    pub ibc_timeout_height_offset: u64,
    /// relay_whitelist_duration specifies the number of blocks before a relay can
    /// be started by any relayer. Prior to this duration, only the whitelisted
    /// relayers can start the relay.
    #[prost(int64, tag="3")]
    pub relay_whitelist_duration: i64,
    /// relay_expiry_duration specifies the number of blocks after which a relay
    /// that has not been started will be pruned from the store
    #[prost(int64, tag="4")]
    pub relay_expiry_duration: i64,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MsgSetBridgeEnabled {
    #[prost(string, tag="1")]
    pub creator: ::prost::alloc::string::String,
    #[prost(uint64, tag="2")]
    pub bridge_id: u64,
    #[prost(bool, tag="3")]
    pub is_enabled: bool,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MsgSetBridgeEnabledResponse {
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MsgUpdateAxelarIbcChannel {
    #[prost(string, tag="1")]
    pub creator: ::prost::alloc::string::String,
    #[prost(string, tag="2")]
    pub channel_id: ::prost::alloc::string::String,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MsgUpdateAxelarIbcChannelResponse {
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MsgUpdateIbcTimeoutHeightOffset {
    #[prost(string, tag="1")]
    pub creator: ::prost::alloc::string::String,
    #[prost(uint64, tag="2")]
    pub offset: u64,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MsgUpdateIbcTimeoutHeightOffsetResponse {
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MsgUpdateRelayExpiry {
    #[prost(string, tag="1")]
    pub creator: ::prost::alloc::string::String,
    #[prost(int64, tag="2")]
    pub expiry: i64,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MsgUpdateRelayExpiryResponse {
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MsgUpdateRelayWhitelistDuration {
    #[prost(string, tag="1")]
    pub creator: ::prost::alloc::string::String,
    #[prost(int64, tag="2")]
    pub whitelist_duration: i64,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MsgUpdateRelayWhitelistDurationResponse {
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MsgUpdateParams {
    #[prost(string, tag="1")]
    pub authority: ::prost::alloc::string::String,
    #[prost(message, optional, tag="2")]
    pub params: ::core::option::Option<Params>,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MsgUpdateParamsResponse {
}
/// MsgAxelarSendToken is a convenience method to send a *Axelar Supported* token
/// via axelar.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MsgAxelarSendToken {
    /// for this message, the message creator will be the sender of the token
    #[prost(string, tag="1")]
    pub creator: ::prost::alloc::string::String,
    /// the destination chain. see axelar for list of supported chain names:
    /// <https://docs.axelar.dev/dev/reference/mainnet-chain-names>
    #[prost(string, tag="2")]
    pub destination_chain: ::prost::alloc::string::String,
    /// the address on destination chain
    #[prost(string, tag="3")]
    pub destination_address: ::prost::alloc::string::String,
    /// see supported tokens: <https://docs.axelar.dev/resources/mainnet#assets,>
    /// convert them to ibc equivalent on carbon
    #[prost(message, optional, tag="4")]
    pub tokens: ::core::option::Option<::cosmos_sdk_proto::cosmos::base::v1beta1::Coin>,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MsgAxelarSendTokenResponse {
}
/// MsgAxelarCallContract is a convenience method to do a GMP call to axelar.
/// This method will allow you do a `callContract` without having to specify the
/// following: TypeGeneralMessageWithToken, IBC channel, IBC port, AxelarGMPAcc
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MsgAxelarCallContract {
    /// for this message, the message creator will be the sender
    #[prost(string, tag="1")]
    pub creator: ::prost::alloc::string::String,
    /// the destination chain. see axelar for list of supported chain names:
    /// <https://docs.axelar.dev/dev/reference/mainnet-chain-names>
    #[prost(string, tag="2")]
    pub destination_chain: ::prost::alloc::string::String,
    /// the address on destination chain
    #[prost(string, tag="3")]
    pub destination_address: ::prost::alloc::string::String,
    /// abi encoded bytes TODO: give abi encoding example?
    #[prost(bytes="vec", tag="4")]
    pub payload: ::prost::alloc::vec::Vec<u8>,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MsgAxelarCallContractResponse {
}
/// MsgAxelarCallContractWithToken is a convenience method to do a GMP call to
/// axelar and attach some *Axelar Supported* tokens This method will allow you
/// do a `callContractWithToken` without having to specify the following:
/// TypeGeneralMessageWithToken, IBC channel, IBC port, AxelarGMPAcc
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MsgAxelarCallContractWithToken {
    /// for this message, the message creator will be the sender
    #[prost(string, tag="1")]
    pub creator: ::prost::alloc::string::String,
    /// the destination chain. see axelar for list of supported chain names:
    /// <https://docs.axelar.dev/dev/reference/mainnet-chain-names>
    #[prost(string, tag="2")]
    pub destination_chain: ::prost::alloc::string::String,
    /// the address on destination chain
    #[prost(string, tag="3")]
    pub destination_address: ::prost::alloc::string::String,
    /// see supported tokens: <https://docs.axelar.dev/resources/mainnet#assets,>
    /// convert them to ibc equivalent on carbon
    #[prost(message, optional, tag="4")]
    pub tokens: ::core::option::Option<::cosmos_sdk_proto::cosmos::base::v1beta1::Coin>,
    /// abi encoded bytes TODO: give abi encoding example?
    #[prost(bytes="vec", tag="5")]
    pub payload: ::prost::alloc::vec::Vec<u8>,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MsgAxelarCallContractWithTokenResponse {
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MsgCreateConnection {
    #[prost(string, tag="1")]
    pub creator: ::prost::alloc::string::String,
    #[prost(uint64, tag="2")]
    pub bridge_id: u64,
    #[prost(string, tag="3")]
    pub chain_id: ::prost::alloc::string::String,
    #[prost(string, tag="4")]
    pub chain_display_name: ::prost::alloc::string::String,
    #[prost(string, tag="5")]
    pub token_gateway_address: ::prost::alloc::string::String,
    #[prost(string, tag="6")]
    pub encoding: ::prost::alloc::string::String,
    #[prost(bool, tag="7")]
    pub is_enabled: bool,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MsgCreateConnectionResponse {
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MsgUpdateConnection {
    #[prost(string, tag="1")]
    pub creator: ::prost::alloc::string::String,
    #[prost(string, tag="2")]
    pub connection_id: ::prost::alloc::string::String,
    #[prost(message, optional, tag="3")]
    pub update_connection_params: ::core::option::Option<UpdateConnectionParams>,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct UpdateConnectionParams {
    #[prost(message, optional, tag="1")]
    pub chain_display_name: ::core::option::Option<::pbjson_types::StringValue>,
    #[prost(message, optional, tag="2")]
    pub is_enabled: ::core::option::Option<::pbjson_types::BoolValue>,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MsgUpdateConnectionResponse {
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MsgRemoveConnection {
    #[prost(string, tag="1")]
    pub creator: ::prost::alloc::string::String,
    #[prost(string, tag="2")]
    pub connection_id: ::prost::alloc::string::String,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MsgRemoveConnectionResponse {
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MsgAddControllersForConnection {
    #[prost(string, tag="1")]
    pub creator: ::prost::alloc::string::String,
    #[prost(message, optional, tag="2")]
    pub controllers: ::core::option::Option<ControllerContracts>,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MsgAddControllersForConnectionResponse {
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MsgUpdateControllersForConnection {
    #[prost(string, tag="1")]
    pub creator: ::prost::alloc::string::String,
    #[prost(string, tag="2")]
    pub connection_id: ::prost::alloc::string::String,
    #[prost(message, optional, tag="3")]
    pub controllers: ::core::option::Option<ControllersToUpdate>,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MsgUpdateControllersForConnectionResponse {
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MsgRemoveControllersForConnection {
    #[prost(string, tag="1")]
    pub creator: ::prost::alloc::string::String,
    #[prost(string, tag="2")]
    pub connection_id: ::prost::alloc::string::String,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MsgRemoveControllersForConnectionResponse {
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MsgRemoveNonceMapForConnection {
    #[prost(string, tag="1")]
    pub creator: ::prost::alloc::string::String,
    #[prost(string, tag="2")]
    pub connection_id: ::prost::alloc::string::String,
    #[prost(string, tag="3")]
    pub gateway_address: ::prost::alloc::string::String,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MsgRemoveNonceMapForConnectionResponse {
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MsgRegisterExternalToken {
    #[prost(string, tag="1")]
    pub creator: ::prost::alloc::string::String,
    #[prost(string, tag="2")]
    pub connection_id: ::prost::alloc::string::String,
    #[prost(string, tag="3")]
    pub asset_address: ::prost::alloc::string::String,
    #[prost(message, optional, tag="4")]
    pub decimals: ::core::option::Option<::pbjson_types::Int64Value>,
    #[prost(string, tag="5")]
    pub carbon_token_name: ::prost::alloc::string::String,
    #[prost(message, optional, tag="6")]
    pub relay_fee: ::core::option::Option<::cosmos_sdk_proto::cosmos::base::v1beta1::Coin>,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MsgRegisterExternalTokenResponse {
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MsgDeregisterExternalToken {
    #[prost(string, tag="1")]
    pub creator: ::prost::alloc::string::String,
    #[prost(string, tag="2")]
    pub connection_id: ::prost::alloc::string::String,
    #[prost(string, tag="3")]
    pub denom: ::prost::alloc::string::String,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MsgDeregisterExternalTokenResponse {
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MsgDeployNativeToken {
    #[prost(string, tag="1")]
    pub creator: ::prost::alloc::string::String,
    #[prost(string, tag="2")]
    pub connection_id: ::prost::alloc::string::String,
    #[prost(string, tag="3")]
    pub denom: ::prost::alloc::string::String,
    #[prost(message, optional, tag="4")]
    pub relay_fee: ::core::option::Option<::cosmos_sdk_proto::cosmos::base::v1beta1::Coin>,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MsgDeployNativeTokenResponse {
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MsgRegisterDeployedToken {
    #[prost(string, tag="1")]
    pub creator: ::prost::alloc::string::String,
    #[prost(string, tag="2")]
    pub connection_id: ::prost::alloc::string::String,
    #[prost(string, tag="3")]
    pub token_address: ::prost::alloc::string::String,
    #[prost(string, tag="4")]
    pub denom: ::prost::alloc::string::String,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MsgRegisterDeployedTokenResponse {
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MsgWithdrawToken {
    #[prost(string, tag="1")]
    pub creator: ::prost::alloc::string::String,
    #[prost(string, tag="2")]
    pub connection_id: ::prost::alloc::string::String,
    #[prost(string, tag="3")]
    pub receiver: ::prost::alloc::string::String,
    #[prost(message, optional, tag="4")]
    pub tokens: ::core::option::Option<::cosmos_sdk_proto::cosmos::base::v1beta1::Coin>,
    #[prost(message, optional, tag="5")]
    pub relay_fee: ::core::option::Option<::cosmos_sdk_proto::cosmos::base::v1beta1::Coin>,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MsgWithdrawTokenResponse {
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MsgUpdateExternalToken {
    #[prost(string, tag="1")]
    pub creator: ::prost::alloc::string::String,
    #[prost(string, tag="2")]
    pub connection_id: ::prost::alloc::string::String,
    #[prost(string, tag="3")]
    pub asset_address: ::prost::alloc::string::String,
    #[prost(string, tag="4")]
    pub token_name: ::prost::alloc::string::String,
    #[prost(int64, tag="5")]
    pub decimals: i64,
    #[prost(bool, tag="6")]
    pub is_carbon_owned: bool,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MsgUpdateExternalTokenResponse {
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MsgDeleteExternalToken {
    #[prost(string, tag="1")]
    pub creator: ::prost::alloc::string::String,
    #[prost(string, tag="2")]
    pub connection_id: ::prost::alloc::string::String,
    #[prost(string, tag="3")]
    pub denom: ::prost::alloc::string::String,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MsgDeleteExternalTokenResponse {
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MsgExecuteFromCarbon {
    #[prost(string, tag="1")]
    pub creator: ::prost::alloc::string::String,
    #[prost(string, tag="2")]
    pub connection_id: ::prost::alloc::string::String,
    #[prost(string, tag="3")]
    pub execution_contract: ::prost::alloc::string::String,
    #[prost(string, tag="4")]
    pub method: ::prost::alloc::string::String,
    #[prost(bytes="vec", tag="5")]
    pub execution_bytes: ::prost::alloc::vec::Vec<u8>,
    #[prost(message, optional, tag="6")]
    pub tokens: ::core::option::Option<::cosmos_sdk_proto::cosmos::base::v1beta1::Coin>,
    #[prost(message, optional, tag="7")]
    pub relay_fee: ::core::option::Option<::cosmos_sdk_proto::cosmos::base::v1beta1::Coin>,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MsgExecuteFromCarbonResponse {
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MsgStartRelay {
    #[prost(string, tag="1")]
    pub relayer: ::prost::alloc::string::String,
    #[prost(uint64, tag="2")]
    pub nonce: u64,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MsgStartRelayResponse {
}
// @@protoc_insertion_point(module)
