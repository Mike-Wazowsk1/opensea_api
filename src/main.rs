use self::schema::info::dsl::*;
use self::schema::tokens::dsl::*;

use actix_web::{get, web, App, HttpRequest, HttpResponse, HttpServer, Responder};
use diesel::associations::HasTable;
use diesel::pg::PgConnection;
use diesel::prelude::*;
use diesel::r2d2::{self, ConnectionManager};
use dotenvy::dotenv;
use ethers::prelude::gas_oracle::GasNow;
use ethers::prelude::gas_oracle::GasOracleMiddleware;
use ethers::prelude::*;
use ethers::providers::{Http, Provider};
use once_cell::sync::Lazy;
use opensea_api::models::{InfoPoint, NewToken, Token};
use opensea_api::*;
use std::collections::HashSet;
use std::str::FromStr;
use std::sync::Arc;
use std::time::Duration;
use std::time::Instant;
use std::{collections::HashMap, error::Error};
use std::{env, thread};
use tokio::task;
mod structs;
abigen!(
    NftContract,
    "abi.json",
    event_derives(serde::Deserialize, serde::Serialize)
);
// use tokio::task::JoinSet;

type Client = GasOracleMiddleware<Provider<Http>, GasNow>;

const MATICURL: &str = "https://polygon-rpc.com";
static mut GLOBAL_OWNERS: Lazy<HashSet<String>> = Lazy::new(|| HashSet::new());
// pub const MIGRATIONS: EmbeddedMigrations = embed_migrations!();

pub async fn establish_connection() -> PgConnection {
    dotenv().ok();
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    PgConnection::establish(&database_url)
        .unwrap_or_else(|_| panic!("Error connecting to {}", database_url))
}

async fn make_nft_array(connection: &mut PgConnection) -> Vec<structs::TokenLocal> {
    let mut result: Vec<structs::TokenLocal> = vec![];
    let mut db: Vec<Token> = tokens.load(connection).expect("Need data");
    db.sort_by(|a, b| a.index.partial_cmp(&b.index).unwrap());
    for l in db {
        let tmp = structs::TokenLocal {
            index: l.index,
            id: l.id.unwrap(),
            bracket: l.bracket.unwrap(),
            level: l.level.unwrap(),
            count: l.count.unwrap(),
        };
        result.push(tmp);
    }
    result
}

async fn get_counts(
    client: &Client,
    contract_addr: &H160,
    address: &web::Path<String>,
    nfts: &mut Vec<structs::TokenLocal>,
) -> Vec<U256> {
    let contract: NftContract<&GasOracleMiddleware<Provider<Http>, GasNow>> =
        NftContract::new(contract_addr.clone(), Arc::new(client.clone()));
    let mut ids: Vec<U256> = vec![];
    let mut addresses: Vec<Address> = vec![];

    for tok in &mut *nfts {
        let tmp = match U256::from_str_radix(&tok.id, 10) {
            Ok(x) => x,
            Err(_e) => {
                continue;
            }
        };
        ids.push(tmp);
    }
    for _i in 0..ids.len() {
        let tmp = match Address::from_str(&address) {
            Ok(x) => x,
            Err(_x) => {
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
    nfts: &mut Vec<structs::TokenLocal>,
) -> Vec<U256> {
    let contract = NftContract::new(contract_addr.clone(), Arc::new(client.clone()));
    let mut ids: Vec<U256> = vec![];
    let mut addresses: Vec<Address> = vec![];

    for tok in &mut *nfts {
        let tmp = match U256::from_str_radix(&tok.id, 10) {
            Ok(x) => x,
            Err(_e) => {
                continue;
            }
        };
        ids.push(tmp);
    }
    for _i in 0..ids.len() {
        let tmp = match Address::from_str(&address) {
            Ok(x) => x,
            Err(_x) => {
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

async fn get_collection_from_opensea() -> Result<structs::NFTResponse, Box<dyn Error>> {
    let client = reqwest::Client::builder().build()?;

    let resp = client
        .get("https://api.opensea.io/v2/collection/bitgesell-road/nfts?limit=50")
        .header("accept", "application/json")
        .header("X-API-KEY", "71ddd979592c4a1ab3a3c4e9a1d6924c")
        .send()
        .await?
        .text()
        .await?;
    let nfts: structs::NFTResponse = serde_json::from_str(&resp)?;
    Ok(nfts)
}

#[get("/info")]
async fn get_nfts() -> impl Responder {
    match get_collection_from_opensea().await {
        Ok(nfts) => HttpResponse::Ok()
            .append_header(("Access-Control-Allow-Origin", "*"))
            .json(nfts),
        Err(err) => HttpResponse::InternalServerError().body(err.to_string()),
    }
}

async fn multiplicator(tokens_arr: &Vec<structs::TokenLocal>) -> Vec<f64> {
    let mut multiply = vec![1.; 20];
    let mut cur = 0;
    //Common
    if tokens_arr[0].count > 0
        && tokens_arr[1].count > 0
        && tokens_arr[2].count > 0
        && tokens_arr[3].count > 0
    {
        multiply[cur] = 1.5;
    }
    cur += 1;

    if tokens_arr[4].count > 0
        && tokens_arr[5].count > 0
        && tokens_arr[6].count > 0
        && tokens_arr[7].count > 0
    {
        multiply[cur] = 1.5;
    }
    cur += 1;
    //Special
    let mut i = 8;
    while i <= 20 {
        if tokens_arr[i].count > 0 && tokens_arr[i + 1].count > 0 && tokens_arr[i + 2].count > 0 {
            multiply[cur] = 2.;
        }
        i += 3;
        cur += 1
    }
    //Rare
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

async fn get_pts(tokens_arr: &Vec<structs::TokenLocal>) -> f64 {
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
async fn get_pts_by_grade(tokens_arr: &Vec<structs::TokenLocal>) -> HashMap<String, f64> {
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
async fn get_nft_by_address(
    address: web::Path<String>,
    pool: web::Data<r2d2::Pool<ConnectionManager<PgConnection>>>,
) -> impl Responder {
    // let connection = &mut establish_connection().await;
    let mut connection = pool.get().unwrap();

    let provider = Provider::<Http>::try_from(MATICURL).unwrap();
    let gas_oracle = GasNow::new();
    let client: GasOracleMiddleware<Provider<Http>, GasNow> =
        GasOracleMiddleware::new(provider, gas_oracle);
    let mut nfts: Vec<structs::TokenLocal> = make_nft_array(&mut connection).await;

    let tmp_a = &address.clone();
    if tmp_a == "0x289140cbe1cb0b17c7e0d83f64a1852f67215845" {
        let mut res: Vec<structs::TokenLocalTmp> = Vec::new();
        for token_local in &nfts {
            // Create structs::TokenLocalTmp with the calculated value of is_full
            let token_local_tmp = structs::TokenLocalTmp {
                index: token_local.index,
                count: token_local.count,
                id: token_local.id.clone(),
                bracket: token_local.bracket,
                level: token_local.level.clone(),
                is_full: false,
            };

            res.push(token_local_tmp);
        }

        let response: structs::Fun1Response = structs::Fun1Response {
            nfts: res,
            sum_pts: 0.0,
            pts_by_grade: HashMap::new(),
        };
        return HttpResponse::Ok()
            .append_header(("Access-Control-Allow-Origin", "*"))
            .json(response);
    }

    let contract_addr = Address::from_str("0x2953399124F0cBB46d2CbACD8A89cF0599974963").unwrap();

    let _balance = get_counts(&client, &contract_addr, &address, &mut nfts).await;
    let sum_pts = get_pts(&nfts).await;
    let pts_by_grade = get_pts_by_grade(&nfts).await;
    let mut res: Vec<structs::TokenLocalTmp> = Vec::new();

    for token_local in &nfts {
        let bracket_tmp = token_local.bracket;
        let is_full = nfts
            .iter()
            .filter(|&t| t.bracket == bracket_tmp)
            .all(|t| t.count > 0);

        // Create structs::TokenLocalTmp with the calculated value of is_full
        let token_local_tmp = structs::TokenLocalTmp {
            index: token_local.index,
            count: token_local.count,
            id: token_local.id.clone(),
            bracket: token_local.bracket,
            level: token_local.level.clone(),
            is_full,
        };

        res.push(token_local_tmp);
    }
    res.sort_by(|a, b| a.index.partial_cmp(&b.index).unwrap());

    let response: structs::Fun1Response = structs::Fun1Response {
        nfts: res,
        sum_pts,
        pts_by_grade,
    };

    HttpResponse::Ok()
        .append_header(("Access-Control-Allow-Origin", "*"))
        .json(response)
}
//
async fn get_nft_by_address_local(
    nfts: &mut Vec<structs::TokenLocal>,
    address: &String,
    client: &GasOracleMiddleware<Provider<Http>, GasNow>,
    contract_addr: &H160,
) -> f64 {
    let _balance = get_counts_local(&client, &contract_addr, &address, nfts).await;
    let pts: f64 = get_pts(&nfts).await;
    pts
}

async fn get_ids(connection: &mut PgConnection) -> (Vec<String>, Vec<structs::TokenLocal>) {
    // let mut connection = pool.get().unwrap();
    // let mut blocked = false;
    let mut token_ids = Vec::new();
    // let connection = &mut establish_connection().await;
    let nfts: Vec<structs::TokenLocal> = make_nft_array(connection).await;

    // let nfts_t = match get_collection_from_opensea().await {
    //     Ok(x) => x,
    //     Err(_x) => {
    //         blocked = true;
    //         structs::NFTResponse { nfts: Vec::new() }
    //     }
    // };
    // if !blocked {
    //     for n in nfts_t.nfts {
    //         let token_id = match n.identifier {
    //             Some(x) => x,
    //             None => continue,
    //         };
    //         token_ids.push(token_id);
    //     }
    // } else {
    for n in &nfts {
        token_ids.push((*n.id).to_string());
        // }
    }

    (token_ids, nfts)
}

#[get("/get_owners")]
async fn get_owners(
    req: HttpRequest,
    pool: web::Data<r2d2::Pool<ConnectionManager<PgConnection>>>,
) -> impl Responder {
    let q: String = req.query_string().replace("&", " ").replace("=", " ");
    let query: Vec<&str> = q.split(" ").collect();
    let mut connection = pool.get().unwrap();
    // let start_time = Instant::now();

    // let provider = Provider::<Http>::try_from(MATICURL).unwrap();

    // let gas_oracle = GasNow::new();
    // let client: GasOracleMiddleware<Provider<Http>, GasNow> =
    //     GasOracleMiddleware::new(provider, gas_oracle);
    let contract_addr = Address::from_str("0x2953399124F0cBB46d2CbACD8A89cF0599974963").unwrap();

    let mut limit = 0;
    let mut page = 0;
    let mut search = "".to_string();

    for i in 0..query.len() {
        if query[i] == "limit" {
            limit = i32::from_str_radix(query[i + 1], 10).unwrap()
        }
        if query[i] == "page" {
            page = i32::from_str_radix(query[i + 1], 10).unwrap()
        }
        if query[i] == "match" || query[i] == "search" {
            search = query[i + 1].to_lowercase()
        }
    }

    // let provider = Provider::<Http>::try_from(MATICURL).unwrap();
    // let key = env::var("PRIVATE_KEY").unwrap();
    // let wallet: LocalWallet = key
    //     .parse::<LocalWallet>()
    //     .unwrap()
    //     .with_chain_id(Chain::Moonbeam);
    // let gas_oracle = GasNow::new();
    // let client: GasOracleMiddleware<Provider<Http>, GasNow> = GasOracleMiddleware::new(provider, gas_oracle);
    // let tup = get_ids(&mut connection).await;
    // let mut nfts: Vec<structs::TokenLocal> = tup.1;

    let mut scores: HashMap<String, f64> = HashMap::new();

    // let client_clone = client;

    unsafe {
        let mut tasks = Vec::new();

        for owner in GLOBAL_OWNERS.iter() {
            let ok_owner = owner.clone();
            if !scores.contains_key(&ok_owner) {
                let provider = Provider::<Http>::try_from(MATICURL).unwrap();

                let gas_oracle = GasNow::new();
                let client: GasOracleMiddleware<Provider<Http>, GasNow> =
                    GasOracleMiddleware::new(provider, gas_oracle);
                // let nfts = &nfts;
                let tup = get_ids(&mut connection).await;
                let mut nfts: Vec<structs::TokenLocal> = tup.1;
                let task = task::spawn(async move {
                    let current_address =
                        get_nft_by_address_local(&mut nfts, &ok_owner, &client, &contract_addr)
                            .await;
                    let current_pts = current_address;
                    (ok_owner, current_pts)
                });
                tasks.push(task);
            }
        }

        for task in tasks {
            let (ok_owner, current_pts) = task.await.unwrap();
            scores.insert(ok_owner, current_pts);
        }
    }

    let mut sorted_scores: Vec<(&String, &f64)> = scores.iter().collect();

    sorted_scores.sort_by(|a, b| {
        let score_comparison = b.1.partial_cmp(a.1).unwrap();
        if score_comparison == std::cmp::Ordering::Equal {
            a.0.partial_cmp(b.0).unwrap()
        } else {
            score_comparison
        }
    });
    let mut s = 0.;
    for st in &sorted_scores {
        s += st.1;
    }

    let mut result = Vec::new();

    for i in 0..sorted_scores.len() {
        let reward = (wbgl(&mut connection).await * sorted_scores[i].1) / s;
        if search == "" {
            result.push(structs::Fun2Response {
                address: sorted_scores[i].0.to_string(),
                score: *sorted_scores[i].1,
                reward,
            });
        } else {
            if sorted_scores[i].0.to_string() == search {
                result.push(structs::Fun2Response {
                    address: sorted_scores[i].0.to_string(),
                    score: *sorted_scores[i].1,
                    reward,
                });
            }
        }
    }
    if search != "" {
        return HttpResponse::Ok()
            .append_header(("Access-Control-Allow-Origin", "*"))
            .json(result);
    }
    let mut final_result = Vec::new();

    page = page - 1;
    let cur_index: i32 = limit * page as i32;
    let mut j = 0;
    if limit == 0 {
        limit = sorted_scores.len() as i32;
    }
    // let connection: &mut PgConnection = &mut establish_connection().await;

    for i in cur_index as usize..sorted_scores.len() {
        let reward = (wbgl(&mut connection).await * sorted_scores[i].1) / s;
        final_result.push(structs::Fun2Response {
            address: sorted_scores[i].0.to_string(),
            score: *sorted_scores[i].1,
            reward,
        });
        j += 1;
        if j == limit {
            break;
        }
    }

    HttpResponse::Ok()
        .append_header(("Access-Control-Allow-Origin", "*"))
        .json(final_result)
}

async fn get_owners_local() {
    let connection: &mut PgConnection = &mut establish_connection().await;

    loop {
        let tup = get_ids(connection).await;
        let token_ids = tup.0;

        for tok in token_ids {
            if tok == "NO_VALUE".to_string() {
                continue;
            }

            let url = format!(
                "https://polygon-mainnet.g.alchemy.com/nft/v2/lUgTmkM2_xJvUIF0dB1iFt0IQrqd4Haw/getOwnersForToken?contractAddress=0x2953399124F0cBB46d2CbACD8A89cF0599974963&tokenId={tok}",
                tok = tok
            );
            let resp = reqwest::get(url).await;
            let tmp_resp = match resp {
                Ok(x) => x,
                Err(x) => {
                    println!("Can't make request to alchemy {:?}", x);
                    continue;
                }
            };
            let resp_text = tmp_resp.text().await.unwrap();

            let tmp_serde: Result<structs::OwnersResponse, serde_json::Error> =
                serde_json::from_str(&resp_text);
            let tmp_owners: structs::OwnersResponse = match tmp_serde {
                Ok(x) => x,
                Err(_x) => structs::OwnersResponse {
                    owners: Vec::new(),
                    page_key: Option::None,
                },
            };
            println!("{:?}", tmp_owners);

            for owner in tmp_owners.owners {
                let ok_owner = match owner {
                    Some(x) => x,
                    None => continue,
                };
                unsafe {
                    if !GLOBAL_OWNERS.contains(&ok_owner) {
                        GLOBAL_OWNERS.insert(ok_owner);
                    }
                }
            }
        }
        thread::sleep(Duration::from_millis(300000));
    }
}

#[get("/get_last_trade")]
async fn get_last_trade(
    pool: web::Data<r2d2::Pool<ConnectionManager<PgConnection>>>,
) -> impl Responder {
    let mut connection = pool.get().unwrap();
    let value = info.load::<InfoPoint>(&mut connection).unwrap();
    let last_tx = value[0].hash.clone();

    // let tup = get_ids().await;
    // let token_ids = tup.0;
    // let mut max: U256 = U256::zero();
    // let mut last_tx = String::new();

    // unsafe {
    //     for owner in GLOBAL_OWNERS.iter() {
    //         let tx_url =
    //             "https://polygon-mainnet.g.alchemy.com/v2/lUgTmkM2_xJvUIF0dB1iFt0IQrqd4Haw";

    //         let payload = RequestPayload {
    //             id: 1,
    //             jsonrpc: "2.0".to_string(),
    //             method: "alchemy_getAssetTransfers".to_string(),
    //             params: vec![RequestParam {
    //                 from_block: "0x0".to_string(),
    //                 to_block: "latest".to_string(),
    //                 to_address: owner.to_string(),
    //                 category: vec!["external".to_string(), "erc1155".to_string()],
    //                 with_metadata: false,
    //                 exclude_zero_value: true,
    //                 max_count: "0x3e8".to_string(),
    //                 contract_addresses: vec![
    //                     "0x2953399124F0cBB46d2CbACD8A89cF0599974963".to_string()
    //                 ],
    //             }],
    //         };
    //         let client = reqwest::Client::new();
    //         let response = client
    //             .post(tx_url)
    //             .json(&payload)
    //             .header("accept", "application/json")
    //             .header("content-type", "application/json")
    //             .send()
    //             .await;

    //         let response_text = response.unwrap().text().await.unwrap();
    //         let trnasfers: Result<TxHistoryResponse, serde_json::Error> =
    //             serde_json::from_str(&response_text);
    //         let history = match trnasfers {
    //             Ok(x) => x,
    //             Err(x) => {
    //                 println!("Error matching TxHistoryResponse {} {}", x, response_text);
    //                 continue;
    //             }
    //         };
    //         let result = match history.result {
    //             Some(x) => x,
    //             None => continue,
    //         };
    //         let transfers = result.transfers;

    //         for tr in transfers {
    //             match tr.erc1155_metadata {
    //                 Some(x) => {
    //                     let mut cur_tokens: Vec<String> = Vec::new();
    //                     for t in x {
    //                         let token_id = match t.token_id {
    //                             Some(x) => {
    //                                 let without_prefix = x.trim_start_matches("0x");
    //                                 let z = match U256::from_str_radix(without_prefix, 16) {
    //                                     Ok(x) => x,
    //                                     Err(x) => {
    //                                         println!("Error parse blockNum {}", x);
    //                                         continue;
    //                                     }
    //                                 };
    //                                 format!("{}", z)
    //                             }
    //                             None => continue,
    //                         };
    //                         cur_tokens.push(token_id);
    //                     }
    //                     let mut contains = false;
    //                     'outer: for ct in &cur_tokens {
    //                         for ti in &token_ids {
    //                             if ti == ct {
    //                                 contains = true;
    //                                 break 'outer;
    //                             }
    //                         }
    //                     }

    //                     if contains {
    //                         let cur_block = match tr.block_num {
    //                             Some(x) => {
    //                                 let without_prefix = x.trim_start_matches("0x");
    //                                 let z = match U256::from_str_radix(without_prefix, 16) {
    //                                     Ok(x) => x,
    //                                     Err(x) => {
    //                                         println!("Error parse blockNum {}", x);
    //                                         continue;
    //                                     }
    //                                 };
    //                                 z
    //                             }
    //                             None => continue,
    //                         };
    //                         if cur_block > max {
    //                             match tr.hash {
    //                                 Some(x) => {
    //                                     last_tx = x;
    //                                     max = cur_block;
    //                                 }
    //                                 None => continue,
    //                             };
    //                         }
    //                     }
    //                 }
    //                 None => continue,
    //             }
    //         }
    //     }
    // }
    let href = format!("https://bscscan.com/tx/{last_tx}");

    let response = structs::LastTradeResponse {
        hash: last_tx,
        href,
    };
    HttpResponse::Ok()
        .append_header(("Access-Control-Allow-Origin", "*"))
        .json(response)
}

#[get("/get_wbgl")]
async fn get_wbgl(pool: web::Data<r2d2::Pool<ConnectionManager<PgConnection>>>) -> impl Responder {
    let mut conn = pool.get().unwrap();
    // let connection: &mut PgConnection = &mut establish_connection().await;

    // let wbgl

    HttpResponse::Ok()
        .append_header(("Access-Control-Allow-Origin", "*"))
        .json(wbgl(&mut conn).await)
}

#[get("/get_pages/{limit}")]
async fn get_pages(limit: web::Path<i32>) -> impl Responder {
    let z: i32;

    unsafe {
        let a = GLOBAL_OWNERS.len() as i32;
        let b = limit.into_inner();
        if a % b == 0 {
            z = a / b;
        } else {
            z = a / b + 1;
        }
    }
    HttpResponse::Ok()
        .append_header(("Access-Control-Allow-Origin", "*"))
        .json(z)
}
async fn wbgl(connection: &mut PgConnection) -> f64 {
    // let connection = &mut establish_connection().await;
    // let mut conn: &mut PgConnection = connection.clone();
    let value = info.load::<InfoPoint>(connection).unwrap();
    value[0].wbgl.unwrap() as f64
}

#[get("/get_payment")]
async fn get_payment(
    pool: web::Data<r2d2::Pool<ConnectionManager<PgConnection>>>,
) -> impl Responder {
    let start_time = Instant::now();

    let mut connection = pool.get().unwrap();

    // let provider = Provider::<Http>::try_from(MATICURL).unwrap();

    // let gas_oracle = GasNow::new();
    // let client: GasOracleMiddleware<Provider<Http>, GasNow> =
    //     GasOracleMiddleware::new(provider, gas_oracle);
    let contract_addr = Address::from_str("0x2953399124F0cBB46d2CbACD8A89cF0599974963").unwrap();

    // let tup = get_ids(&mut connection).await;
    println!("get_ids: {:?}", start_time.elapsed());

    // let mut nfts: Vec<structs::TokenLocal> = tup.1;

    let mut scores: HashMap<String, f64> = HashMap::new();

    // let client_clone = client;

    unsafe {
        let mut tasks = Vec::new();

        for owner in GLOBAL_OWNERS.iter() {
            let ok_owner = owner.clone();
            if !scores.contains_key(&ok_owner) {
                let provider = Provider::<Http>::try_from(MATICURL).unwrap();

                let gas_oracle = GasNow::new();
                let client: GasOracleMiddleware<Provider<Http>, GasNow> =
                    GasOracleMiddleware::new(provider, gas_oracle);
                // let nfts = &nfts;
                let tup = get_ids(&mut connection).await;
                let mut nfts: Vec<structs::TokenLocal> = tup.1;
                let task = task::spawn(async move {
                    let current_address =
                        get_nft_by_address_local(&mut nfts, &ok_owner, &client, &contract_addr)
                            .await;
                    let current_pts = current_address;
                    (ok_owner, current_pts)
                });
                tasks.push(task);
            }
        }

        for task in tasks {
            let (ok_owner, current_pts) = task.await.unwrap();
            scores.insert(ok_owner, current_pts);
        }
    }
    println!("get_nft_by_address_local: {:?}", start_time.elapsed());

    let mut sorted_scores: Vec<(&String, &f64)> = scores.iter().collect();

    sorted_scores.sort_by(|a, b| {
        let score_comparison = b.1.partial_cmp(a.1).unwrap();
        if score_comparison == std::cmp::Ordering::Equal {
            a.0.partial_cmp(b.0).unwrap()
        } else {
            score_comparison
        }
    });
    println!("sort_by: {:?}", start_time.elapsed());

    let mut s = 0.;
    for st in &sorted_scores {
        s += st.1;
    }
    let mut result: Vec<String> = Vec::new();
    for i in 0..sorted_scores.len() {
        let reward = (wbgl(&mut connection).await * sorted_scores[i].1) / s;
        println!("reward: {:?}", start_time.elapsed());

        let str_reward = format!("{}", reward);
        result.push(format!("{}?{}", sorted_scores[i].0, str_reward));
    }
    let text = result.join(";");
    println!("result: {:?}", start_time.elapsed());

    HttpResponse::Ok()
        .append_header(("Access-Control-Allow-Origin", "*"))
        .json(text)
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    tokio::spawn(async {
        get_owners_local().await;
    });
    // Loading .env into environment variable.
    dotenv::dotenv().ok();

    // set up database connection pool
    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL");
    let manager = ConnectionManager::<PgConnection>::new(database_url);
    let pool: r2d2::Pool<ConnectionManager<PgConnection>> = r2d2::Pool::builder()
        .test_on_check_out(true)
        .build(manager)
        .expect("Could not build connection pool");

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(pool.clone()))
            .route("/", web::get().to(|| async { "Actix REST API" }))
            .service(get_nfts)
            .service(get_nft_by_address)
            .service(get_owners)
            .service(init_db)
            .service(get_wbgl)
            .service(get_last_trade)
            .service(get_pages)
            .service(get_payment)
    })
    .bind("0.0.0.0:8080")?
    .run()
    .await
}

// #[allow(dead_code)]
// // #[get("/owners")]
// async fn get_owners_old() -> impl Responder {
//     let url = "https://polygon-mainnet.g.alchemy.com/nft/v2/lUgTmkM2_xJvUIF0dB1iFt0IQrqd4Haw/getOwnersForCollection?contractAddress=0x2953399124F0cBB46d2CbACD8A89cF0599974963&withTokenBalances=false";
//     let response = reqwest::get(url).await.unwrap();
//     let text = response.text().await.unwrap();

//     let owners: Owners = serde_json::from_str(&text).unwrap();
//     let mut scores: HashMap<String, f64> = HashMap::new();
//     // let provider = Provider::<Http>::try_from(MATICURL).unwrap();

//     // let gas_oracle = GasNow::new();
//     // let client: GasOracleMiddleware<Provider<Http>, GasNow> = GasOracleMiddleware::new(provider, gas_oracle);
//     let connection = &mut establish_connection().await;
//     let nfts: Vec<structs::TokenLocal> = make_nft_array(connection).await;
//     let mut set = JoinSet::new();
//     let mut handles = Vec::new();

//     for addr in owners.owner_addresses {
//         let mut nfts_clone: Vec<structs::TokenLocal> = nfts.clone();

//         let handle = set.spawn(async move {
//             let s = match addr {
//                 Some(x) => x,
//                 None => "".to_string(),
//             };

//             let current_tuple = get_nft_by_address_local(&mut nfts_clone, &s).await;
//             (s, current_tuple)
//         });
//         handles.push(handle);
//     }
//     while let Some(res) = set.join_next().await {
//         let (s, current_tuple) = res.unwrap();
//         let pts = current_tuple;
//         if pts == -100. {
//             continue;
//         }
//         if pts >= 0.0 {
//             scores.insert(s.to_string(), pts);
//         }
//     }
//     let v: Vec<(&String, &f64)> = scores.iter().collect();

//     let mut sorted_scores: Vec<(&String, &f64)> = v;
//     sorted_scores.sort_by(|a, b| b.1.partial_cmp(a.1).unwrap());
//     HttpResponse::Ok()
//         .append_header(("Access-Control-Allow-Origin", "*"))
//         .json(sorted_scores)
// }

#[get("/init_db")]
async fn init_db(pool: web::Data<r2d2::Pool<ConnectionManager<PgConnection>>>) -> impl Responder {
    let result: Vec<structs::TokenLocal> = vec![
        structs::TokenLocal {
            index: 0,
            id: "18349153976137682097687065310984821295737582987254388036615603441181132849302"
                .to_string(),
            count: 0,
            bracket: 0,
            level: "Common".to_string(),
        },
        structs::TokenLocal {
            index: 1,
            id: "18349153976137682097687065310984821295737582987254388036615603429086504943816"
                .to_string(),
            count: 0,
            bracket: 0,
            level: "Common".to_string(),
        },
        structs::TokenLocal {
            index: 2,
            id: "18349153976137682097687065310984821295737582987254388036615603443380156104854"
                .to_string(),
            count: 0,
            bracket: 0,
            level: "Common".to_string(),
        },
        structs::TokenLocal {
            index: 3,
            id: "18349153976137682097687065310984821295737582987254388036615603437882597965974"
                .to_string(),
            count: 0,
            bracket: 0,
            level: "Common".to_string(),
        },
        structs::TokenLocal {
            index: 4,
            id: "18349153976137682097687065310984821295737582987254388036615603436783086338198"
                .to_string(),
            count: 0,
            bracket: 1,
            level: "Common".to_string(),
        },
        structs::TokenLocal {
            index: 5,
            id: "18349153976137682097687065310984821295737582987254388036615603442280644477078"
                .to_string(),
            count: 0,
            bracket: 1,
            level: "Common".to_string(),
        },
        structs::TokenLocal {
            index: 6,
            id: "NO_VALUE".to_string(),
            count: 0,
            bracket: 1,
            level: "Common".to_string(),
        },
        structs::TokenLocal {
            index: 7,
            id: "18349153976137682097687065310984821295737582987254388036615603418091388666006"
                .to_string(),
            count: 0,
            bracket: 1,
            level: "Common".to_string(),
        },
        structs::TokenLocal {
            index: 8,
            id: "18349153976137682097687065310984821295737582987254388036615603451076737499211"
                .to_string(),
            count: 0,
            bracket: 2,
            level: "Special".to_string(),
        },
        structs::TokenLocal {
            index: 9,
            id: "18349153976137682097687065310984821295737582987254388036615603432385039827019"
                .to_string(),
            count: 0,
            bracket: 2,
            level: "Special".to_string(),
        },
        structs::TokenLocal {
            index: 10,
            id: "18349153976137682097687065310984821295737582987254388036615603444479667732555"
                .to_string(),
            count: 0,
            bracket: 2,
            level: "Special".to_string(),
        },
        structs::TokenLocal {
            index: 11,
            id: "NO_VALUE".to_string(),
            count: 0,
            bracket: 3,
            level: "Special".to_string(),
        },
        structs::TokenLocal {
            index: 12,
            id: "18349153976137682097687065310984821295737582987254388036615603445579179360331"
                .to_string(),
            count: 0,
            bracket: 3,
            level: "Special".to_string(),
        },
        structs::TokenLocal {
            index: 13,
            id: "NO_VALUE".to_string(),
            count: 0,
            bracket: 3,
            level: "Special".to_string(),
        },
        structs::TokenLocal {
            index: 14,
            id: "NO_VALUE".to_string(),
            count: 0,
            bracket: 4,
            level: "Special".to_string(),
        },
        structs::TokenLocal {
            index: 15,
            id: "18349153976137682097687065310984821295737582987254388036615603452176249126987"
                .to_string(),
            count: 0,
            bracket: 4,
            level: "Special".to_string(),
        },
        structs::TokenLocal {
            index: 16,
            id: "NO_VALUE".to_string(),
            count: 0,
            bracket: 4,
            level: "Special".to_string(),
        },
        structs::TokenLocal {
            index: 17,
            id: "NO_VALUE".to_string(),
            count: 0,
            bracket: 5,
            level: "Special".to_string(),
        },
        structs::TokenLocal {
            index: 18,
            id: "NO_VALUE".to_string(),
            count: 0,
            bracket: 5,
            level: "Special".to_string(),
        },
        structs::TokenLocal {
            index: 19,
            id: "NO_VALUE".to_string(),
            count: 0,
            bracket: 5,
            level: "Special".to_string(),
        },
        structs::TokenLocal {
            index: 20,
            id: "NO_VALUE".to_string(),
            count: 0,
            bracket: 6,
            level: "Special".to_string(),
        },
        structs::TokenLocal {
            index: 21,
            id: "NO_VALUE".to_string(),
            count: 0,
            bracket: 6,
            level: "Special".to_string(),
        },
        structs::TokenLocal {
            index: 22,
            id: "NO_VALUE".to_string(),
            count: 0,
            bracket: 6,
            level: "Special".to_string(),
        },
        structs::TokenLocal {
            index: 23,
            id: "18349153976137682097687065310984821295737582987254388036615603420290411921433"
                .to_string(),
            count: 0,
            bracket: 7,
            level: "Rare".to_string(),
        },
        structs::TokenLocal {
            index: 24,
            id: "NO_VALUE".to_string(),
            count: 0,
            bracket: 7,
            level: "Rare".to_string(),
        },
        structs::TokenLocal {
            index: 25,
            id: "18349153976137682097687065310984821295737582987254388036615603448877714243609"
                .to_string(),
            count: 0,
            bracket: 8,
            level: "Rare".to_string(),
        },
        structs::TokenLocal {
            index: 26,
            id: "NO_VALUE".to_string(),
            count: 0,
            bracket: 8,
            level: "Rare".to_string(),
        },
        structs::TokenLocal {
            index: 27,
            id: "18349153976137682097687065310984821295737582987254388036615603446678690988057"
                .to_string(),
            count: 0,
            bracket: 9,
            level: "Rare".to_string(),
        },
        structs::TokenLocal {
            index: 28,
            id: "18349153976137682097687065310984821295737582987254388036615603449977225871385"
                .to_string(),
            count: 0,
            bracket: 9,
            level: "Rare".to_string(),
        },
        structs::TokenLocal {
            index: 29,
            id: "18349153976137682097687065310984821295737582987254388036615603447778202615833"
                .to_string(),
            count: 0,
            bracket: 10,
            level: "Rare".to_string(),
        },
        structs::TokenLocal {
            index: 30,
            id: "NO_VALUE".to_string(),
            count: 0,
            bracket: 10,
            level: "Rare".to_string(),
        },
        structs::TokenLocal {
            index: 31,
            id: "NO_VALUE".to_string(),
            count: 0,
            bracket: 11,
            level: "Rare".to_string(),
        },
        structs::TokenLocal {
            index: 32,
            id: "18349153976137682097687065310984821295737582987254388036615603435683574710297"
                .to_string(),
            count: 0,
            bracket: 11,
            level: "Rare".to_string(),
        },
        structs::TokenLocal {
            index: 33,
            id: "NO_VALUE".to_string(),
            count: 0,
            bracket: 15,
            level: "Unique".to_string(),
        },
        structs::TokenLocal {
            index: 34,
            id: "NO_VALUE".to_string(),
            count: 0,
            bracket: 15,
            level: "Unique".to_string(),
        },
        structs::TokenLocal {
            index: 35,
            id: "NO_VALUE".to_string(),
            count: 0,
            bracket: 15,
            level: "Unique".to_string(),
        },
        structs::TokenLocal {
            index: 36,
            id: "NO_VALUE".to_string(),
            count: 0,
            bracket: 15,
            level: "Unique".to_string(),
        },
        structs::TokenLocal {
            index: 37,
            id: "18349153976137682097687065310984821295737582987254388036615603438982109593601"
                .to_string(),
            count: 0,
            bracket: 15,
            level: "Legendary".to_string(),
        },
    ];

    let mut connection = pool.get().unwrap();

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
            .get_result(&mut connection)
            .expect("Error saving new post");
    }
    HttpResponse::Ok()
        .append_header(("Access-Control-Allow-Origin", "*"))
        .json("Oke")
}
