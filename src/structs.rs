use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use serde;

#[derive(Serialize, Deserialize, Debug)]
pub struct NFT {
    pub identifier: Option<String>,
    pub collection: Option<String>,
    pub contract: Option<String>,
    pub token_standard: Option<String>,
    pub name: Option<String>,
    pub description: Option<String>,
    pub image_url: Option<String>,
    pub metadata_url: Option<String>,
    pub created_at: Option<String>,
    pub updated_at: Option<String>,
    pub is_disabled: bool,
    pub is_nsfw: bool,
}



#[derive(Debug, Deserialize, Serialize)]
#[allow(dead_code)]
pub struct NFTResponse {
    pub nfts: Vec<NFT>,
}
#[derive(Debug, Deserialize, Serialize, Clone)]
#[allow(dead_code)]
pub struct TokenLocal {
    pub index: i32,
    pub count: i32,
    pub id: String,
    pub bracket: i32,
    pub level: String,
}
#[derive(Debug, Deserialize, Serialize, Clone)]
#[allow(dead_code)]
pub struct TokenLocalTmp {
    pub index: i32,
    pub count: i32,
    pub id: String,
    pub bracket: i32,
    pub level: String,
    pub is_full: bool,
}
#[derive(Debug, Deserialize, Serialize)]
#[allow(dead_code)]
pub struct Tx {
    #[serde(rename = "blockNumber")]
    pub block_number: Option<String>,
    #[serde(rename = "timeStamp")]
    pub time_stamp: Option<String>,
    pub hash: Option<String>,
    pub nonce: Option<String>,
    #[serde(rename = "blockHash")]
    pub block_hash: Option<String>,
    pub from: Option<String>,
    #[serde(rename = "contractAddress")]
    pub contract_address: Option<String>,
    pub to: Option<String>,
    #[serde(rename = "tokenID")]
    pub token_id: Option<String>,
    #[serde(rename = "tokenName")]
    pub token_name: Option<String>,
    #[serde(rename = "tokens_arrymbol")]
    pub token_symbol: Option<String>,
    #[serde(rename = "tokenDecimal")]
    pub token_decimal: Option<String>,
    #[serde(rename = "transactionIndex")]
    pub transaction_index: Option<String>,
    pub gas: Option<String>,
    #[serde(rename = "gasPrice")]
    pub gas_price: Option<String>,
    #[serde(rename = "gasUsed")]
    pub gas_used: Option<String>,
    #[serde(rename = "cumulativeGasUsed")]
    pub cumulative_gas_used: Option<String>,
    pub input: Option<String>,
    pub confirmations: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
#[allow(dead_code)]
pub struct Owners {
    #[serde(rename = "ownerAddresses")]
    pub owner_addresses: Vec<Option<String>>,
    #[serde(rename = "pageKey")]
    pub page_key: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
#[allow(dead_code)]
pub struct OwnersResponse {
    pub owners: Vec<Option<String>>,
    #[serde(rename = "pageKey")]
    pub page_key: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
#[allow(dead_code)]
pub struct ScanerResponse {
    status: Option<String>,
    message: Option<String>,
    result: Vec<Tx>,
}

#[derive(Debug, Deserialize, Serialize)]
#[allow(dead_code)]
pub struct Fun1Response {
    pub nfts: Vec<TokenLocalTmp>,
    pub sum_pts: f64,
    pub pts_by_grade: HashMap<String, f64>,
}

#[derive(Debug, Deserialize, Serialize)]
#[allow(dead_code)]
pub struct Fun2Response {
    pub address: String,
    pub score: f64,
    pub reward: i64,
}

#[derive(Serialize, Deserialize)]
#[allow(dead_code)]
pub struct RequestPayload {
    pub id: u8,
    pub jsonrpc: String,
    pub method: String,
    pub params: Vec<RequestParam>,
}

#[derive(Serialize, Deserialize)]
#[allow(dead_code)]
pub struct RequestParam {
    #[serde(rename = "fromBlock")]
    from_block: String,
    #[serde(rename = "toBlock")]
    to_block: String,
    #[serde(rename = "toAddress")]
    to_address: String,
    category: Vec<String>,
    #[serde(rename = "withMetadata")]
    with_metadata: bool,
    #[serde(rename = "excludeZeroValue")]
    exclude_zero_value: bool,
    #[serde(rename = "maxCount")]
    max_count: String,
    #[serde(rename = "contractAddresses")]
    contract_addresses: Vec<String>,
}

#[derive(Deserialize)]
#[allow(dead_code)]
pub struct Transfer {
    category: Option<String>,
    #[serde(rename = "blockNum")]
    block_num: Option<String>,
    from: Option<String>,
    to: Option<String>,
    value: Option<f64>,

    #[serde(rename = "erc721TokenId")]
    erc721_token_id: Option<String>,

    #[serde(rename = "erc1155Metadata")]
    erc1155_metadata: Option<Vec<Erc1155Metadata>>,
    #[serde(rename = "tokenId")]
    token_id: Option<String>,
    asset: Option<String>,
    #[serde(rename = "uniqueId")]
    unique_id: Option<String>,
    hash: Option<String>,
    #[serde(rename = "rawContract")]
    raw_contract: RawContract,
    metadata: Option<Metadata>,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
pub struct Erc1155Metadata {
    #[serde(rename = "tokenId")]
    pub token_id: Option<String>,
    pub value: Option<String>,
}

#[derive(Deserialize)]
#[allow(dead_code)]
pub struct RawContract {
    value: Option<String>,
    address: Option<String>,
    decimal: Option<String>,
}

#[derive(Deserialize)]
#[allow(dead_code)]
pub struct Metadata {
    #[serde(rename = "blockTimestamp")]
    block_timestamp: Option<String>,
}

#[derive(Deserialize)]
#[allow(dead_code)]
pub struct Response {
    #[serde(rename = "pageKey")]
    pub page_key: Option<String>,
    pub transfers: Vec<Transfer>,
}

#[derive(Deserialize)]
#[allow(dead_code)]
pub struct TxHistoryResponse {
    id: i32,
    jsonrpc: Option<String>,
    result: Option<Response>,
}

#[derive(Serialize)]
#[allow(dead_code)]
pub struct LastTradeResponse {
    pub hash: String,
    pub href: String,
}

