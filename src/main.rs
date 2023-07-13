use self::schema::tokens::dsl::*;
use actix_web::{get, web, App, HttpResponse, HttpServer, Responder};
use diesel::associations::HasTable;
use diesel::pg::PgConnection;
use diesel::prelude::*;
use dotenvy::dotenv;
use ethers::prelude::*;
use ethers::providers::{Http, Provider};
use opensea_api::models::{NewToken, Token};
use opensea_api::*;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::env;
use std::str::FromStr;
use std::sync::Arc;
use std::{collections::HashMap, error::Error};
use tokio::task::JoinSet;

type Client = SignerMiddleware<Provider<Http>, Wallet<k256::ecdsa::SigningKey>>;

#[derive(Serialize, Deserialize, Debug)]
struct NFT {
    identifier: Option<String>,
    collection: Option<String>,
    contract: Option<String>,
    token_standard: Option<String>,
    name: Option<String>,
    description: Option<String>,
    image_url: Option<String>,
    metadata_url: Option<String>,
    created_at: Option<String>,
    updated_at: Option<String>,
    is_disabled: bool,
    is_nsfw: bool,
}

abigen!(
    NftContract,
    "abi.json",
    event_derives(serde::Deserialize, serde::Serialize)
);

#[derive(Debug, Deserialize, Serialize)]
struct NFTResponse {
    nfts: Vec<NFT>,
}
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct TokenLocal {
    index: i32,
    count: i32,
    id: String,
    bracket: i32,
    level: String,
}
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct TokenLocalTmp {
    index: i32,
    count: i32,
    id: String,
    bracket: i32,
    level: String,
    is_full: bool,
}
#[derive(Debug, Deserialize, Serialize)]
struct Tx {
    #[serde(rename = "blockNumber")]
    block_number: Option<String>,
    #[serde(rename = "timeStamp")]
    time_stamp: Option<String>,
    hash: Option<String>,
    nonce: Option<String>,
    #[serde(rename = "blockHash")]
    block_hash: Option<String>,
    from: Option<String>,
    #[serde(rename = "contractAddress")]
    contract_address: Option<String>,
    to: Option<String>,
    #[serde(rename = "tokenID")]
    token_id: Option<String>,
    #[serde(rename = "tokenName")]
    token_name: Option<String>,
    #[serde(rename = "tokens_arrymbol")]
    token_symbol: Option<String>,
    #[serde(rename = "tokenDecimal")]
    token_decimal: Option<String>,
    #[serde(rename = "transactionIndex")]
    transaction_index: Option<String>,
    gas: Option<String>,
    #[serde(rename = "gasPrice")]
    gas_price: Option<String>,
    #[serde(rename = "gasUsed")]
    gas_used: Option<String>,
    #[serde(rename = "cumulativeGasUsed")]
    cumulative_gas_used: Option<String>,
    input: Option<String>,
    confirmations: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
struct Owners {
    #[serde(rename = "ownerAddresses")]
    owner_addresses: Vec<Option<String>>,
    #[serde(rename = "pageKey")]
    page_key: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
struct OwnersResponse {
    owners: Vec<Option<String>>,
    #[serde(rename = "pageKey")]
    page_key: Option<String>,
}

const MATICURL: &str = "https://polygon-rpc.com";
#[derive(Debug, Deserialize, Serialize)]
struct ScanerResponse {
    status: Option<String>,
    message: Option<String>,
    result: Vec<Tx>,
}

#[derive(Debug, Deserialize, Serialize)]
struct Fun1Response {
    nfts: Vec<TokenLocalTmp>,
    sum_pts: f64,
    pts_by_grade: HashMap<String, f64>,
}

#[derive(Debug, Deserialize, Serialize)]
struct Fun2Response {
    address: String,
    score: f64,
    reward: f64,
}

#[derive(Serialize, Deserialize)]
struct RequestPayload {
    id: u8,
    jsonrpc: String,
    method: String,
    params: Vec<RequestParam>,
}

#[derive(Serialize, Deserialize)]
struct RequestParam {
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

#[derive(Debug, Deserialize)]
struct Transfer {
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
struct Erc1155Metadata {
    #[serde(rename = "tokenId")]
    token_id: Option<String>,
    value: Option<String>,
}

#[derive(Debug, Deserialize)]
struct RawContract {
    value: Option<String>,
    address: Option<String>,
    decimal: Option<String>,
}

#[derive(Debug, Deserialize)]
struct Metadata {
    #[serde(rename = "blockTimestamp")]
    block_timestamp: Option<String>,
}

#[derive(Debug, Deserialize)]
struct Response {
    #[serde(rename = "pageKey")]
    page_key: Option<String>,
    transfers: Vec<Transfer>,
}

#[derive(Debug, Deserialize)]
struct TxHistoryResponse {
    id: i32,
    jsonrpc: Option<String>,
    result: Option<Response>,
}

#[derive(Debug, Serialize)]
struct LastTradeResponse{
    hash:String,
    block:U256,
    href:String
}

pub async fn establish_connection() -> PgConnection {
    dotenv().ok();
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    PgConnection::establish(&database_url)
        .unwrap_or_else(|_| panic!("Error connecting to {}", database_url))
}

async fn make_nft_array(connection: &mut PgConnection) -> Vec<TokenLocal> {
    let mut result: Vec<TokenLocal> = vec![];
    let db: Vec<Token> = tokens.load(connection).expect("Need data");
    for l in db {
        let tmp = TokenLocal {
            index: l.index,
            id: l.id.unwrap(),
            bracket: l.bracket.unwrap(),
            level: l.level.unwrap(),
            count: l.count.unwrap(),
        };
        result.push(tmp);
    }
    // let nfts = get_nft(&address).await;
    // // println!("{:?}", nfts);
    // let tmp = address.to_string().to_lowercase();
    // for nft in nfts.result {
    //     let address_to = match nft.to {
    //         Some(x) => x.to_lowercase(),
    //         None => continue,
    //     };
    //     if tmp == address_to {
    //         for mut t in &mut result {
    //             if t.id
    //                 == match nft.token_id.clone() {
    //                     Some(x) => x,
    //                     None => {
    //                         continue;
    //                     }
    //                 }
    //             {
    //                 t.count += 1;
    //             }
    //         }
    //     }
    // }
    result
}

async fn get_counts(
    client: &Client,
    contract_addr: &H160,
    address: &web::Path<String>,
    nfts: &mut Vec<TokenLocal>,
) -> Vec<U256> {
    let contract = NftContract::new(contract_addr.clone(), Arc::new(client.clone()));
    let mut ids: Vec<U256> = vec![];
    let mut addresses: Vec<Address> = vec![];

    for tok in &mut *nfts {
        let tmp = match U256::from_str_radix(&tok.id, 10) {
            Ok(x) => {
                println!("All Ok! {:?} {:?}", x, tok);
                x
            }
            Err(_e) => {
                println!("Can't parse:{}", _e);
                continue;
            }
        };
        ids.push(tmp);
    }
    for _i in 0..ids.len() {
        let tmp = match Address::from_str(&address) {
            Ok(x) => x,
            Err(x) => {
                println!("Error: {}", x);
                continue;
            }
        };
        addresses.push(tmp);
    }

    let balance = match contract
        .balance_of_batch(addresses, ids.clone())
        .call()
        .await
    {
        Ok(x) => x,
        Err(_x) => return Vec::new(),
    };
    for i in 0..ids.len() {
        for j in 0..nfts.len() {
            if ids[i].to_string() == nfts[j].id {
                nfts[j].count = balance[i].as_u32() as i32;
            }
        }
    }
    balance
}

async fn get_counts_local(
    client: &Client,
    contract_addr: &H160,
    address: &String,
    nfts: &mut Vec<TokenLocal>,
) -> Vec<U256> {
    let contract = NftContract::new(contract_addr.clone(), Arc::new(client.clone()));
    let mut ids: Vec<U256> = vec![];
    let mut addresses: Vec<Address> = vec![];

    for tok in &mut *nfts {
        let tmp = match U256::from_str_radix(&tok.id, 10) {
            Ok(x) => x,
            Err(_e) => {
                println!("Can't parse: {}", _e);
                continue;
            }
        };
        ids.push(tmp);
    }
    for _i in 0..ids.len() {
        let tmp = match Address::from_str(&address) {
            Ok(x) => x,
            Err(x) => {
                println!("Error: {}", x);
                continue;
            }
        };
        addresses.push(tmp);
    }

    let balance = match contract
        .balance_of_batch(addresses, ids.clone())
        .call()
        .await
    {
        Ok(x) => x,
        Err(_x) => return Vec::new(),
    };
    for i in 0..ids.len() {
        for j in 0..nfts.len() {
            if ids[i].to_string() == nfts[j].id {
                nfts[j].count = balance[i].as_u32() as i32;
            }
        }
    }
    balance
}

async fn get_collection_from_opensea() -> Result<NFTResponse, Box<dyn Error>> {
    // let proxy = reqwest::Proxy::http("http://202.40.177.69:80")?;
    let client = reqwest::Client::builder().build()?;

    let resp = client
        .get("https://api.opensea.io/v2/collection/bitgesell-road/nfts?limit=50")
        .header("accept", "application/json")
        .header("X-API-KEY", "71ddd979592c4a1ab3a3c4e9a1d6924c")
        .send()
        .await?
        .text()
        .await?;
    println!("string {:#?}", resp);
    let nfts: NFTResponse = serde_json::from_str(&resp)?;
    println!("NFTS: {:#?} , Len: {}", nfts, nfts.nfts.len());
    Ok(nfts)
}

#[get("/info")]
async fn get_nfts() -> impl Responder {
    match get_collection_from_opensea().await {
        Ok(nfts) => HttpResponse::Ok().append_header(("ACCESS_CONTROL_ALLOW_ORIGIN", "*")).json(nfts),
        Err(err) => HttpResponse::InternalServerError().body(err.to_string()),
    }
}

async fn multiplicator(tokens_arr: &Vec<TokenLocal>) -> Vec<f64> {
    let mut multiply = vec![1.; 12];
    let mut cur = 0;
    //Common
    if tokens_arr[0].count > 0
        && tokens_arr[1].count > 0
        && tokens_arr[2].count > 0
        && tokens_arr[3].count > 0
    {
        multiply[cur] = 1.5;
        cur += 1;
    }
    if tokens_arr[4].count > 0
        && tokens_arr[5].count > 0
        && tokens_arr[6].count > 0
        && tokens_arr[7].count > 0
    {
        multiply[cur] = 1.5;
        cur += 1;
    }
    let mut i = 8;
    while i <= 20 {
        if tokens_arr[i].count > 0 && tokens_arr[i + 1].count > 0 && tokens_arr[i + 2].count > 0 {
            multiply[cur] = 2.;
        }
        i += 3;
        cur += 1
    }
    i = 23;
    while i <= 31 {
        if tokens_arr[i].count > 0 && tokens_arr[i + 1].count > 0 {
            multiply[cur] = 3.;
        }
        i += 2;
        cur += 1
    }
    multiply
}

async fn get_pts(tokens_arr: &Vec<TokenLocal>) -> f64 {
    let points: HashMap<&str, f64> = HashMap::from([
        ("Common", 1.),
        ("Special", 3.),
        ("Rare", 7.),
        ("Unique", 30.),
        ("Legendary", 50.),
    ]);

    let mut pts = 0.;
    let coef = multiplicator(tokens_arr).await;
    for token in tokens_arr {
        let lvl = token.level.as_str();
        let point = match points.get(&lvl) {
            Some(x) => x,
            None => &1.,
        };
        pts += coef[token.bracket as usize] * point * token.count as f64;
    }
    pts
}
async fn get_pts_by_grade(tokens_arr: &Vec<TokenLocal>) -> HashMap<String, f64> {
    let points: HashMap<&str, f64> = HashMap::from([
        ("Common", 1.),
        ("Special", 3.),
        ("Rare", 7.),
        ("Unique", 30.),
        ("Legendary", 50.),
    ]);
    let mut scores: HashMap<String, f64> = HashMap::from([
        ("Common".to_string(), 0.0),
        ("Special".to_string(), 0.),
        ("Rare".to_string(), 0.),
        ("Unique".to_string(), 0.),
        ("Legendary".to_string(), 0.),
    ]);

    let coef = multiplicator(tokens_arr).await;
    for g in ["Common", "Special", "Rare", "Unique", "Legendary"] {
        let mut pts = 0.;
        for token in tokens_arr {
            let lvl = token.level.as_str();
            let point = match points.get(&lvl) {
                Some(x) => x,
                None => &1.,
            };
            if lvl == g {
                pts += coef[token.bracket as usize] * point * token.count as f64;
                scores.entry(lvl.to_string()).and_modify(|x| *x = pts);
            }
        }
    }
    scores
}

#[get("/nft/{address}")]
async fn get_nft_by_address(address: web::Path<String>) -> impl Responder {
    let connection = &mut establish_connection().await;

    let provider = Provider::<Http>::try_from(MATICURL).unwrap();
    let key: Result<String, env::VarError> = env::var("PRIVATE_KEY");
    let wallet: LocalWallet = key
        .unwrap()
        .parse::<LocalWallet>()
        .unwrap()
        .with_chain_id(Chain::Moonbeam);
    let client = SignerMiddleware::new(provider.clone(), wallet.clone());

    let mut nfts: Vec<TokenLocal> = make_nft_array(connection).await;

    let contract_addr = Address::from_str("0x2953399124F0cBB46d2CbACD8A89cF0599974963").unwrap();

    let _balance = get_counts(&client, &contract_addr, &address, &mut nfts).await;
    let sum_pts = get_pts(&nfts).await;
    let pts_by_grade = get_pts_by_grade(&nfts).await;
    let mut res: Vec<TokenLocalTmp> = Vec::new();

    for token_local in &nfts {
        let bracket_tmp = token_local.bracket;
        let is_full = nfts
            .iter()
            .filter(|&t| t.bracket == bracket_tmp)
            .all(|t| t.count > 0);

        // Create TokenLocalTmp with the calculated value of is_full
        let token_local_tmp = TokenLocalTmp {
            index: token_local.index,
            count: token_local.count,
            id: token_local.id.clone(),
            bracket: token_local.bracket,
            level: token_local.level.clone(),
            is_full,
        };

        res.push(token_local_tmp);
    }

    let response: Fun1Response = Fun1Response {
        nfts: res,
        sum_pts,
        pts_by_grade,
    };

    HttpResponse::Ok().append_header(("ACCESS_CONTROL_ALLOW_ORIGIN", "*")).json(response)
}
//
async fn get_nft_by_address_local(
    client: &SignerMiddleware<Provider<Http>, Wallet<k256::ecdsa::SigningKey>>,
    nfts: &mut Vec<TokenLocal>,
    address: &String,
) -> f64 {
    let contract_addr = Address::from_str("0x2953399124F0cBB46d2CbACD8A89cF0599974963").unwrap();

    let _balance = get_counts_local(&client, &contract_addr, &address, nfts).await;
    let pts: f64 = get_pts(&nfts).await;
    pts
}

// use tokio::task;

async fn get_ids() -> (Vec<String>, Vec<TokenLocal>) {
    let mut blocked = false;
    let mut token_ids = Vec::new();
    let connection = &mut establish_connection().await;
    let nfts: Vec<TokenLocal> = make_nft_array(connection).await;

    let nfts_t = match get_collection_from_opensea().await {
        Ok(x) => x,
        Err(_x) => {
            println!("opensead blocked. Get Nfts from db");
            blocked = true;
            NFTResponse { nfts: Vec::new() }
        }
    };
    if !blocked {
        for n in nfts_t.nfts {
            let token_id = match n.identifier {
                Some(x) => x,
                None => continue,
            };
            token_ids.push(token_id);
        }
    } else {
        for n in &nfts {
            token_ids.push((*n.id).to_string());
        }
    }

    (token_ids, nfts)
}

#[get("/get_owners")]
async fn get_owners() -> impl Responder {
    let provider = Provider::<Http>::try_from(MATICURL).unwrap();
    let key = env::var("PRIVATE_KEY").unwrap();
    let wallet: LocalWallet = key
        .parse::<LocalWallet>()
        .unwrap()
        .with_chain_id(Chain::Moonbeam);
    let client = SignerMiddleware::new(provider.clone(), wallet.clone());

    let tup = get_ids().await;
    let token_ids = tup.0;
    let mut nfts: Vec<TokenLocal> = tup.1;
    println!("{:?}", nfts);

    let mut scores = HashMap::new();
    // let mut set = JoinSet::new();
    println!("Tokens: {:?}", token_ids);

    for tok in token_ids {
        if tok == "NO_VALUE".to_string() {
            continue;
        }
        let client_clone = client.clone();
        // let scores_clone = scores;

        let url = format!(
                "https://polygon-mainnet.g.alchemy.com/nft/v2/lUgTmkM2_xJvUIF0dB1iFt0IQrqd4Haw/getOwnersForToken?contractAddress=0x2953399124F0cBB46d2CbACD8A89cF0599974963&tokenId={tok}",
                tok = tok
            );
        let resp = reqwest::get(url).await;
        let tmp_resp = match resp {
            Ok(x) => x,
            Err(_x) => panic!("Can't make request to alchemy"),
        };
        let resp_text = tmp_resp.text().await.unwrap();
        println!("tEXT: {}", resp_text);

        let tmp_serde: Result<OwnersResponse, serde_json::Error> = serde_json::from_str(&resp_text);
        let tmp_owners: OwnersResponse = match tmp_serde {
            Ok(x) => x,
            Err(x) => {
                println!("Err {}", x);
                OwnersResponse {
                    owners: Vec::new(),
                    page_key: Option::None,
                }
            }
        };

        for owner in tmp_owners.owners {
            let ok_owner = match owner {
                Some(x) => x,
                None => continue,
            };
            if !scores.contains_key(&ok_owner) {
                let current_address =
                    get_nft_by_address_local(&client_clone, &mut nfts, &ok_owner).await;
                let current_pts = current_address;
                scores.insert(ok_owner, current_pts);
            }
        }
    }

    let mut sorted_scores: Vec<(&String, &f64)> = scores.iter().collect();

    sorted_scores.sort_by(|a, b| b.1.partial_cmp(a.1).unwrap());
    let mut s = 0.;
    for st in &sorted_scores {
        s += st.1;
    }

    let mut result = Vec::new();
    for i in 0..sorted_scores.len() {
        let reward = wbgl().await / (s * sorted_scores[i].1);
        result.push(Fun2Response {
            address: sorted_scores[i].0.to_string(),
            score: *sorted_scores[i].1,
            reward,
        });
    }
    HttpResponse::Ok().append_header(("ACCESS_CONTROL_ALLOW_ORIGIN", "*")).json(result)
}

#[get("/get_last_trade")]
async fn get_last_trade() -> impl Responder {
    let provider = Provider::<Http>::try_from(MATICURL).unwrap();
    let key = env::var("PRIVATE_KEY").unwrap();
    let wallet: LocalWallet = key
        .parse::<LocalWallet>()
        .unwrap()
        .with_chain_id(Chain::Moonbeam);
    let client = SignerMiddleware::new(provider.clone(), wallet.clone());

    let tup = get_ids().await;
    let token_ids = tup.0;
    let mut nfts: Vec<TokenLocal> = tup.1;
    let mut set = HashSet::new();
    let mut max: U256 = U256::zero();
    let mut last_tx = String::new();

    // let mut set = JoinSet::new();

    for tok in &token_ids {
        if *tok == "NO_VALUE".to_string() {
            continue;
        }
        // let scores_clone = scores;
        let url = format!(
                "https://polygon-mainnet.g.alchemy.com/nft/v2/lUgTmkM2_xJvUIF0dB1iFt0IQrqd4Haw/getOwnersForToken?contractAddress=0x2953399124F0cBB46d2CbACD8A89cF0599974963&tokenId={tok}",
                tok = tok
            );
        let resp = reqwest::get(url).await;
        let tmp_resp = match resp {
            Ok(x) => x,
            Err(_x) => panic!("Can't make request to alchemy"),
        };
        let resp_text = tmp_resp.text().await.unwrap();

        let tmp_serde: Result<OwnersResponse, serde_json::Error> = serde_json::from_str(&resp_text);
        let tmp_owners: OwnersResponse = match tmp_serde {
            Ok(x) => x,
            Err(x) => {
                println!("Error MatchOwnerResponse {} {} {}", x, resp_text, tok);
                OwnersResponse {
                    owners: Vec::new(),
                    page_key: Option::None,
                }
            }
        };

        for owner in tmp_owners.owners {
            let ok_owner = match owner {
                Some(x) => x,
                None => continue,
            };
            if !set.contains(&ok_owner) {
                set.insert(ok_owner);
            }
        }
    }
    for owner in set.iter() {
        let tx_url = "https://polygon-mainnet.g.alchemy.com/v2/lUgTmkM2_xJvUIF0dB1iFt0IQrqd4Haw";

        let payload = RequestPayload {
            id: 1,
            jsonrpc: "2.0".to_string(),
            method: "alchemy_getAssetTransfers".to_string(),
            params: vec![RequestParam {
                from_block: "0x0".to_string(),
                to_block: "latest".to_string(),
                to_address: owner.to_string(),
                category: vec!["external".to_string(), "erc1155".to_string()],
                with_metadata: false,
                exclude_zero_value: true,
                max_count: "0x3e8".to_string(),
                contract_addresses: vec!["0x2953399124F0cBB46d2CbACD8A89cF0599974963".to_string()],
            }],
        };
        let client = reqwest::Client::new();
        let response = client
            .post(tx_url)
            .json(&payload)
            .header("accept", "application/json")
            .header("content-type", "application/json")
            .send()
            .await;

        let response_text = response.unwrap().text().await.unwrap();
        let trnasfers: Result<TxHistoryResponse, serde_json::Error> =
            serde_json::from_str(&response_text);
        let history = match trnasfers {
            Ok(x) => x,
            Err(x) => {
                println!("Error matching TxHistoryResponse {} {}", x, response_text);
                continue;
            }
        };
        let result = match history.result {
            Some(x) => x,
            None => continue,
        };
        let transfers = result.transfers;

        for tr in transfers {
            match tr.erc1155_metadata {
                Some(x) => {
                    let mut cur_tokens: Vec<String> = Vec::new();
                    for t in x {
                        let token_id = match t.token_id {
                            Some(x) => {
                                let without_prefix = x.trim_start_matches("0x");
                                let z = match U256::from_str_radix(without_prefix, 16) {
                                    Ok(x) => x,
                                    Err(x) => {
                                        println!("Error parse blockNum {}", x);
                                        continue;
                                    }
                                };
                                format!("{}", z)
                            }
                            None => continue,
                        };
                        cur_tokens.push(token_id);
                    }
                    let mut contains = false;
                    'outer: for ct in &cur_tokens {
                        for ti in &token_ids {
                            if ti == ct {
                                contains = true;
                                break 'outer;
                            }
                        }
                    }

                    if contains {
                        let cur_block = match tr.block_num {
                            Some(x) => {
                                let without_prefix = x.trim_start_matches("0x");
                                let z = match U256::from_str_radix(without_prefix, 16) {
                                    Ok(x) => x,
                                    Err(x) => {
                                        println!("Error parse blockNum {}", x);
                                        continue;
                                    }
                                };
                                z
                            }
                            None => continue,
                        };
                        if cur_block > max {
                            match tr.hash {
                                Some(x) => {
                                    println!("OKAY? {:?} {:?} {:?}", cur_block, x, owner);
                                    last_tx = x;
                                    max = cur_block;
                                }
                                None => continue,
                            };
                        }
                    }
                }
                None => continue,
            }
        }

    }
    let href = format!("https://polygonscan.com/tx/{last_tx}");
    let response = LastTradeResponse{hash:last_tx,block:max,href};
    HttpResponse::Ok().append_header(("ACCESS_CONTROL_ALLOW_ORIGIN", "*")).json(response)
}

#[get("/get_wbgl")]
async fn get_wbgl() -> impl Responder {
    HttpResponse::Ok().append_header(("ACCESS_CONTROL_ALLOW_ORIGIN", "*")).json(wbgl().await)
}

async fn wbgl() -> f64 {
    567.
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();
    let key = env::var("SUKA");
    println!("{:?}", key);

    HttpServer::new(|| {
        App::new()
            .service(get_nfts)
            .service(get_nft_by_address)
            .service(get_owners)
            .service(init_db)
            .service(get_wbgl)
            .service(get_last_trade)
    })
    .bind("0.0.0.0:8080")?
    .run()
    .await
}

#[allow(dead_code)]
// #[get("/owners")]
async fn get_owners_old() -> impl Responder {
    let url = "https://polygon-mainnet.g.alchemy.com/nft/v2/lUgTmkM2_xJvUIF0dB1iFt0IQrqd4Haw/getOwnersForCollection?contractAddress=0x2953399124F0cBB46d2CbACD8A89cF0599974963&withTokenBalances=false";
    let response = reqwest::get(url).await.unwrap();
    let text = response.text().await.unwrap();

    let owners: Owners = serde_json::from_str(&text).unwrap();
    let mut scores: HashMap<String, f64> = HashMap::new();
    let provider = Provider::<Http>::try_from(MATICURL).unwrap();
    let key = env::var("PRIVATE_KEY").unwrap();
    let wallet: LocalWallet = key
        .parse::<LocalWallet>()
        .unwrap()
        .with_chain_id(Chain::Moonbeam);
    let client = SignerMiddleware::new(provider.clone(), wallet.clone());
    let connection = &mut establish_connection().await;
    let nfts: Vec<TokenLocal> = make_nft_array(connection).await;
    let mut set = JoinSet::new();
    let mut handles = Vec::new();
    println!("Len: {:?}", owners.owner_addresses.len());

    for addr in owners.owner_addresses {
        let mut nfts_clone: Vec<TokenLocal> = nfts.clone();

        let client_clone = client.clone();

        let handle = set.spawn(async move {
            let s = match addr {
                Some(x) => x,
                None => {
                    println!("ERROR");
                    "".to_string()
                }
            };

            let current_tuple = get_nft_by_address_local(&client_clone, &mut nfts_clone, &s).await;
            (s, current_tuple)
        });
        handles.push(handle);
    }
    while let Some(res) = set.join_next().await {
        let (s, current_tuple) = res.unwrap();
        let pts = current_tuple;
        if pts == -100. {
            continue;
        }
        if pts >= 0.0 {
            scores.insert(s.to_string(), pts);
        }
    }
    let v: Vec<(&String, &f64)> = scores.iter().collect();

    let mut sorted_scores: Vec<(&String, &f64)> = v;
    sorted_scores.sort_by(|a, b| b.1.partial_cmp(a.1).unwrap());
    HttpResponse::Ok().append_header(("ACCESS_CONTROL_ALLOW_ORIGIN", "*")).json(sorted_scores)
}

#[allow(dead_code)]
async fn get_nft(address: &web::Path<String>) -> ScanerResponse {
    let mut cur_page = 1;
    let mut result: ScanerResponse = ScanerResponse {
        status: None,
        message: None,
        result: Vec::new(),
    };
    let mut tx_array: Vec<Tx> = Vec::new();
    let poly_api = "VTQ4CQUFH8JWB8RIQFST6MT3QVPMIV86Y2".to_string();
    let _eth_api = "JS8AHSJV7H9X1J3MG4J656KM7NAVDA4R1P".to_string();
    let client = match reqwest::Client::builder().build() {
        Ok(client) => client,
        Err(_err) => return result,
    };
    // let mut link = "https://api.etherscan.io/api?module=account&action=tokennfttx&address=0x567A623433D503183Fb383493FdB12A4780e2F60&page=1&offset=100&startblock=0&sort=asc&apikey=YourApiKeyToken".to_string();

    loop {
        let link = format!(
        "https://api.polygonscan.com/api?module=account&action=token1155tx&address={address}&page={cur_page}&offset=100&startblock=0&sort=asc&apikey={poly_api}"
    );

        let resp = match client.get(link).send().await {
            Ok(resp) => resp,
            Err(_err) => continue,
        };

        let resp_text = match resp.text().await {
            Ok(text) => text,
            Err(_err) => continue,
        };
        let tmp: ScanerResponse = serde_json::from_str(&resp_text).unwrap();
        if tmp.result.len() == 0 {
            break;
        }
        println!("TMP: {:?}", tmp);
        for tx in tmp.result {
            tx_array.push(tx);
        }
        // result.result.push(tmp.result.into_iter().next().unwrap());
        cur_page += 1;
    }
    result.result = tx_array;
    result
}

#[get("/init_db")]
async fn init_db() -> impl Responder {
    let result: Vec<TokenLocal> = vec![
        TokenLocal {
            index: 0,
            id: "18349153976137682097687065310984821295737582987254388036615603441181132849302"
                .to_string(),
            count: 0,
            bracket: 0,
            level: "Common".to_string(),
        },
        TokenLocal {
            index: 1,
            id: "18349153976137682097687065310984821295737582987254388036615603429086504943816"
                .to_string(),
            count: 0,
            bracket: 0,
            level: "Common".to_string(),
        },
        TokenLocal {
            index: 2,
            id: "18349153976137682097687065310984821295737582987254388036615603443380156104854"
                .to_string(),
            count: 0,
            bracket: 0,
            level: "Common".to_string(),
        },
        TokenLocal {
            index: 3,
            id: "18349153976137682097687065310984821295737582987254388036615603437882597965974"
                .to_string(),
            count: 0,
            bracket: 0,
            level: "Common".to_string(),
        },
        TokenLocal {
            index: 4,
            id: "18349153976137682097687065310984821295737582987254388036615603436783086338198"
                .to_string(),
            count: 0,
            bracket: 1,
            level: "Common".to_string(),
        },
        TokenLocal {
            index: 5,
            id: "18349153976137682097687065310984821295737582987254388036615603442280644477078"
                .to_string(),
            count: 0,
            bracket: 1,
            level: "Common".to_string(),
        },
        TokenLocal {
            index: 6,
            id: "NO_VALUE".to_string(),
            count: 0,
            bracket: 1,
            level: "Common".to_string(),
        },
        TokenLocal {
            index: 7,
            id: "18349153976137682097687065310984821295737582987254388036615603418091388666006"
                .to_string(),
            count: 0,
            bracket: 1,
            level: "Common".to_string(),
        },
        TokenLocal {
            index: 8,
            id: "18349153976137682097687065310984821295737582987254388036615603451076737499211"
                .to_string(),
            count: 0,
            bracket: 2,
            level: "Special".to_string(),
        },
        TokenLocal {
            index: 9,
            id: "18349153976137682097687065310984821295737582987254388036615603432385039827019"
                .to_string(),
            count: 0,
            bracket: 2,
            level: "Special".to_string(),
        },
        TokenLocal {
            index: 10,
            id: "18349153976137682097687065310984821295737582987254388036615603444479667732555"
                .to_string(),
            count: 0,
            bracket: 2,
            level: "Special".to_string(),
        },
        TokenLocal {
            index: 11,
            id: "NO_VALUE".to_string(),
            count: 0,
            bracket: 3,
            level: "Special".to_string(),
        },
        TokenLocal {
            index: 12,
            id: "18349153976137682097687065310984821295737582987254388036615603445579179360331"
                .to_string(),
            count: 0,
            bracket: 3,
            level: "Special".to_string(),
        },
        TokenLocal {
            index: 13,
            id: "NO_VALUE".to_string(),
            count: 0,
            bracket: 3,
            level: "Special".to_string(),
        },
        TokenLocal {
            index: 14,
            id: "NO_VALUE".to_string(),
            count: 0,
            bracket: 4,
            level: "Special".to_string(),
        },
        TokenLocal {
            index: 15,
            id: "18349153976137682097687065310984821295737582987254388036615603452176249126987"
                .to_string(),
            count: 0,
            bracket: 4,
            level: "Special".to_string(),
        },
        TokenLocal {
            index: 16,
            id: "NO_VALUE".to_string(),
            count: 0,
            bracket: 4,
            level: "Special".to_string(),
        },
        TokenLocal {
            index: 17,
            id: "NO_VALUE".to_string(),
            count: 0,
            bracket: 5,
            level: "Special".to_string(),
        },
        TokenLocal {
            index: 18,
            id: "NO_VALUE".to_string(),
            count: 0,
            bracket: 5,
            level: "Special".to_string(),
        },
        TokenLocal {
            index: 19,
            id: "NO_VALUE".to_string(),
            count: 0,
            bracket: 5,
            level: "Special".to_string(),
        },
        TokenLocal {
            index: 20,
            id: "NO_VALUE".to_string(),
            count: 0,
            bracket: 6,
            level: "Special".to_string(),
        },
        TokenLocal {
            index: 21,
            id: "NO_VALUE".to_string(),
            count: 0,
            bracket: 6,
            level: "Special".to_string(),
        },
        TokenLocal {
            index: 22,
            id: "NO_VALUE".to_string(),
            count: 0,
            bracket: 6,
            level: "Special".to_string(),
        },
        TokenLocal {
            index: 23,
            id: "18349153976137682097687065310984821295737582987254388036615603420290411921433"
                .to_string(),
            count: 0,
            bracket: 7,
            level: "Rare".to_string(),
        },
        TokenLocal {
            index: 24,
            id: "NO_VALUE".to_string(),
            count: 0,
            bracket: 7,
            level: "Rare".to_string(),
        },
        TokenLocal {
            index: 25,
            id: "18349153976137682097687065310984821295737582987254388036615603448877714243609"
                .to_string(),
            count: 0,
            bracket: 8,
            level: "Rare".to_string(),
        },
        TokenLocal {
            index: 26,
            id: "NO_VALUE".to_string(),
            count: 0,
            bracket: 8,
            level: "Rare".to_string(),
        },
        TokenLocal {
            index: 27,
            id: "18349153976137682097687065310984821295737582987254388036615603446678690988057"
                .to_string(),
            count: 0,
            bracket: 9,
            level: "Rare".to_string(),
        },
        TokenLocal {
            index: 28,
            id: "18349153976137682097687065310984821295737582987254388036615603449977225871385"
                .to_string(),
            count: 0,
            bracket: 9,
            level: "Rare".to_string(),
        },
        TokenLocal {
            index: 29,
            id: "18349153976137682097687065310984821295737582987254388036615603447778202615833"
                .to_string(),
            count: 0,
            bracket: 10,
            level: "Rare".to_string(),
        },
        TokenLocal {
            index: 30,
            id: "NO_VALUE".to_string(),
            count: 0,
            bracket: 10,
            level: "Rare".to_string(),
        },
        TokenLocal {
            index: 31,
            id: "NO_VALUE".to_string(),
            count: 0,
            bracket: 11,
            level: "Rare".to_string(),
        },
        TokenLocal {
            index: 32,
            id: "18349153976137682097687065310984821295737582987254388036615603435683574710297"
                .to_string(),
            count: 0,
            bracket: 11,
            level: "Rare".to_string(),
        },
        TokenLocal {
            index: 33,
            id: "NO_VALUE".to_string(),
            count: 0,
            bracket: 11,
            level: "Unique".to_string(),
        },
        TokenLocal {
            index: 34,
            id: "NO_VALUE".to_string(),
            count: 0,
            bracket: 11,
            level: "Unique".to_string(),
        },
        TokenLocal {
            index: 35,
            id: "NO_VALUE".to_string(),
            count: 0,
            bracket: 11,
            level: "Unique".to_string(),
        },
        TokenLocal {
            index: 36,
            id: "NO_VALUE".to_string(),
            count: 0,
            bracket: 11,
            level: "Unique".to_string(),
        },
        TokenLocal {
            index: 37,
            id: "NO_VALUE".to_string(),
            count: 0,
            bracket: 11,
            level: "Legendary".to_string(),
        },
    ];

    let connection = &mut establish_connection().await;

    for token in &result {
        let new_token = NewToken {
            index: &token.index,
            id: &token.id,
            count: &token.count,
            bracket: &token.bracket,
            level: &token.level,
        };

        diesel::insert_into(tokens::table())
            .values(new_token)
            .returning(Token::as_returning())
            .get_result(connection)
            .expect("Error saving new post");
    }
    HttpResponse::Ok().append_header(("ACCESS_CONTROL_ALLOW_ORIGIN", "*")).json("Oke")
}
