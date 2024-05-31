use ethers::addressbook::Address;
use ethers::contract::EthEvent;
use ethers::prelude::{H256, U256};

#[derive(Debug, Clone, PartialEq, Eq, Default, EthEvent)]
#[ethevent(name = "ContractCallApproved", abi = "ContractCallApproved(bytes32,string,string,address,bytes32,bytes32,uint256)")]
pub struct ContractCallApprovedEvent {
    #[ethevent(indexed)]
    pub command_id: H256,
    pub source_chain: String,
    pub source_address: String,
    #[ethevent(indexed)]
    pub contract_address: Address,
    #[ethevent(indexed)]
    pub payload_hash: H256,
    pub source_tx_hash: H256,
    pub source_event_index: U256,
}