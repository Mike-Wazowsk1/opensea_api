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
use reqwest::{self};
use serde::{Deserialize, Serialize};
use std::env;
use std::str::FromStr;
use std::sync::Arc;
use std::time::Instant;
use std::{collections::HashMap, error::Error};


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

async fn get_counts(
    client: &Client,
    contract_addr: &H160,
    address: &web::Path<String>,
    nfts: &Vec<TokenLocal>,
) -> Vec<U256> {
    let contract = NftContract::new(contract_addr.clone(), Arc::new(client.clone()));
    let mut ids: Vec<U256> = vec![];
    let mut addresses: Vec<Address> = vec![];

    for tok in nfts {
        let tmp = match U256::from_str_radix(&tok.id, 10) {
            Ok(x) => {
                println!("All Ok! {:?} {:?}", x, tok);
                x
            }
            Err(_e) => {
                println!("{}", _e);
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

    let balance = match contract.balance_of_batch(addresses, ids).call().await {
        Ok(x) => x,
        Err(x) => return Vec::new(),
    };
    balance
}
async fn get_counts_local(
    client: &Client,
    contract_addr: &H160,
    address: &String,
    nfts: &Vec<TokenLocal>,
) -> Vec<U256> {
    let contract = NftContract::new(contract_addr.clone(), Arc::new(client.clone()));
    let mut ids: Vec<U256> = vec![];
    let mut addresses: Vec<Address> = vec![];

    for tok in nfts {
        let tmp = match U256::from_str_radix(&tok.id, 10) {
            Ok(x) => x,
            Err(_e) => {
                println!("{}", _e);
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
    let balance = match contract.balance_of_batch(addresses, ids).call().await {
        Ok(x) => x,
        Err(x) => return Vec::new(),
    };
    balance
}

#[derive(Debug, Deserialize, Serialize)]
struct NFTResponse {
    nfts: Vec<NFT>,
}
#[derive(Debug, Deserialize, Serialize)]

struct TokenLocal {
    count: i32,
    id: String,
    bracket: i32,
    level: String,
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
    ownerAddresses: Vec<Option<String>>,
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
    nfts: Vec<TokenLocal>,
    pts: f64,
}

pub fn establish_connection() -> PgConnection {
    dotenv().ok();
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    PgConnection::establish(&database_url)
        .unwrap_or_else(|_| panic!("Error connecting to {}", database_url))
}

async fn make_nft_array() -> Vec<TokenLocal> {
    let mut result: Vec<TokenLocal> = vec![];
    let connection = &mut establish_connection();
    let db: Vec<Token> = tokens.load(connection).expect("Need data");
    for l in db {
        let tmp = TokenLocal {
            id: l.id,
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

async fn get_tmp() -> Result<NFTResponse, Box<dyn Error>> {
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
    match get_tmp().await {
        Ok(nfts) => HttpResponse::Ok().json(nfts),
        Err(err) => HttpResponse::InternalServerError().body(err.to_string()),
    }
}

fn multiplicator(tokens_arr: &Vec<TokenLocal>) -> Vec<f64> {
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

fn get_pts(tokens_arr: &Vec<TokenLocal>) -> f64 {
    let points: HashMap<&str, f64> = HashMap::from([
        ("Common", 1.),
        ("Special", 3.),
        ("Rare", 7.),
        ("Unique", 30.),
        ("Legendary", 50.),
    ]);

    let mut pts = 0.;
    let coef = multiplicator(tokens_arr);
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

#[get("/nft/{address}")]
async fn get_nft_by_address(address: web::Path<String>) -> impl Responder {
    // Specify the URL of the Ethereum node you want to connect to

    // Create an HTTP provider
    let provider = Provider::<Http>::try_from(MATICURL).unwrap();
    let key = env::var("PRIVATE_KEY");
    println!("{:?}",key);
    let wallet: LocalWallet = "key"
        .parse::<LocalWallet>()
        .unwrap()
        .with_chain_id(Chain::Moonbeam);
    let client = SignerMiddleware::new(provider.clone(), wallet.clone());

    let mut nfts: Vec<TokenLocal> = make_nft_array().await;

    let contract_addr = Address::from_str("0x2953399124F0cBB46d2CbACD8A89cF0599974963").unwrap();
    // let client = ethers::etherscan::Client::new(Chain::Polygon, MATICURL).unwrap();

    let balance = get_counts(&client, &contract_addr, &address, &nfts).await;
    for i in 0..nfts.len() {
        nfts[i].count = balance[i].as_u32() as i32;
    }
    let pts = get_pts(&nfts);

    let response: Fun1Response = Fun1Response { nfts, pts };

    HttpResponse::Ok().json(response)
}

async fn get_nft_by_address_local(address: &String) -> (Vec<TokenLocal>, f64) {
    // println!("Run Get Nft by adress");
    // Specify the URL of the Ethereum node you want to connect to

    // Create an HTTP provider
    let provider = Provider::<Http>::try_from(MATICURL).unwrap();
    let key = env::var("PRIVATE_KEY").unwrap();
    let wallet: LocalWallet = key
        .parse::<LocalWallet>()
        .unwrap()
        .with_chain_id(Chain::Moonbeam);
    let client = SignerMiddleware::new(provider.clone(), wallet.clone());

    let mut nfts: Vec<TokenLocal> = make_nft_array().await;

    let contract_addr = Address::from_str("0x2953399124F0cBB46d2CbACD8A89cF0599974963").unwrap();
    // let client = ethers::etherscan::Client::new(Chain::Polygon, MATICURL).unwrap();

    let balance = get_counts_local(&client, &contract_addr, &address, &nfts).await;
    if balance.len() != nfts.len() {
        return (Vec::new(), -100.);
    }
    for i in 0..nfts.len() {
        nfts[i].count = balance[i].as_u32() as i32;
    }
    let pts = get_pts(&nfts);
    (nfts, pts)
}

use tokio::task;

use tokio::task::JoinSet;
#[get("/owners")]
async fn get_owners() -> impl Responder {
    let start_time = Instant::now();
    let url = "https://polygon-mainnet.g.alchemy.com/nft/v2/lUgTmkM2_xJvUIF0dB1iFt0IQrqd4Haw/getOwnersForCollection?contractAddress=0x2953399124F0cBB46d2CbACD8A89cF0599974963&withTokenBalances=false";
    let response = reqwest::get(url).await.unwrap();
    let text = response.text().await.unwrap();
    let mut owners: Owners = serde_json::from_str(&text).unwrap();
    let mut scores: HashMap<String, f64> = HashMap::new();
    owners.ownerAddresses.truncate(1000);
    let address_len = owners.ownerAddresses.len();

    let mut set = JoinSet::new();

    println!("{}", &address_len);
    let mut handles = Vec::new();

    for addr in owners.ownerAddresses {
        let s = match addr {
            Some(x) => x,
            None => continue,
        };

        let handle = set.spawn(async move {
            let start_time1 = Instant::now();

            let current_tuple = get_nft_by_address_local(&s).await;
            let elapsed_time1 = start_time1.elapsed();
            let elapsed_time = start_time.elapsed();
            println!("After run fun {},After start {}",elapsed_time1.as_secs_f64(),elapsed_time.as_secs_f64());

            (s, current_tuple)
        });
        handles.push(handle);
    }
    // let hanlers_len = handles.len();

    while let Some(res) = set.join_next().await {
        let (s, current_tuple) = res.unwrap();
        let pts = current_tuple.1;
        if pts == -100. {
            continue;
        }
        if pts >= 0.0 {
            scores.insert(s.to_string(), pts);
        }
    }
    let mut sorted_scores: Vec<(&String, &f64)> = scores.iter().collect();
    sorted_scores.sort_by(|a, b| b.1.partial_cmp(a.1).unwrap());
    let elapsed_time = start_time.elapsed();
    println!("Прошло времени: {} секунд", elapsed_time.as_secs());
    HttpResponse::Ok().json(sorted_scores)
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();
    let key = env::var("SUKA");
    println!("{:?}",key);

    HttpServer::new(|| {
        App::new()
            .service(get_nfts)
            .service(get_nft_by_address)
            .service(get_owners)
            .service(init_db)
    })
    .bind("0.0.0.0:8080")?
    .run()
    .await
}

#[get("/init_db")]
fn init_db() -> impl Responder {
    let result: Vec<TokenLocal> = vec![
        TokenLocal {
            id: "18349153976137682097687065310984821295737582987254388036615603441181132849302"
                .to_string(),
            count: 0,
            bracket: 0,
            level: "Common".to_string(),
        },
        TokenLocal {
            id: "18349153976137682097687065310984821295737582987254388036615603429086504943816"
                .to_string(),
            count: 0,
            bracket: 0,
            level: "Common".to_string(),
        },
        TokenLocal {
            id: "18349153976137682097687065310984821295737582987254388036615603443380156104854"
                .to_string(),
            count: 0,
            bracket: 0,
            level: "Common".to_string(),
        },
        TokenLocal {
            id: "18349153976137682097687065310984821295737582987254388036615603437882597965974"
                .to_string(),
            count: 0,
            bracket: 0,
            level: "Common".to_string(),
        },
        TokenLocal {
            id: "18349153976137682097687065310984821295737582987254388036615603436783086338198"
                .to_string(),
            count: 0,
            bracket: 1,
            level: "Common".to_string(),
        },
        TokenLocal {
            id: "18349153976137682097687065310984821295737582987254388036615603442280644477078"
                .to_string(),
            count: 0,
            bracket: 1,
            level: "Common".to_string(),
        },
        TokenLocal {
            id: "".to_string(),
            count: 0,
            bracket: 1,
            level: "Common".to_string(),
        },
        TokenLocal {
            id: "18349153976137682097687065310984821295737582987254388036615603418091388666006"
                .to_string(),
            count: 0,
            bracket: 1,
            level: "Common".to_string(),
        },
        TokenLocal {
            id: "18349153976137682097687065310984821295737582987254388036615603451076737499211"
                .to_string(),
            count: 0,
            bracket: 2,
            level: "Special".to_string(),
        },
        TokenLocal {
            id: "18349153976137682097687065310984821295737582987254388036615603432385039827019"
                .to_string(),
            count: 0,
            bracket: 2,
            level: "Special".to_string(),
        },
        TokenLocal {
            id: "18349153976137682097687065310984821295737582987254388036615603444479667732555"
                .to_string(),
            count: 0,
            bracket: 2,
            level: "Special".to_string(),
        },
        TokenLocal {
            id: "1".to_string(),
            count: 0,
            bracket: 3,
            level: "Special".to_string(),
        },
        TokenLocal {
            id: "18349153976137682097687065310984821295737582987254388036615603445579179360331"
                .to_string(),
            count: 0,
            bracket: 3,

            level: "Special".to_string(),
        },
        TokenLocal {
            id: "2".to_string(),
            count: 0,
            bracket: 3,

            level: "Special".to_string(),
        },
        TokenLocal {
            id: "3".to_string(),
            count: 0,
            bracket: 4,

            level: "Special".to_string(),
        },
        TokenLocal {
            id: "18349153976137682097687065310984821295737582987254388036615603452176249126987"
                .to_string(),
            count: 0,
            bracket: 4,

            level: "Common".to_string(),
        },
        TokenLocal {
            id: "4".to_string(),
            count: 0,
            bracket: 4,

            level: "Special".to_string(),
        },
        TokenLocal {
            id: "5".to_string(),
            count: 0,
            bracket: 5,

            level: "Special".to_string(),
        },
        TokenLocal {
            id: "6".to_string(),
            count: 0,
            bracket: 5,

            level: "Special".to_string(),
        },
        TokenLocal {
            id: "7".to_string(),
            count: 0,
            bracket: 5,

            level: "Special".to_string(),
        },
        TokenLocal {
            id: "8".to_string(),
            count: 0,
            bracket: 6,

            level: "Special".to_string(),
        },
        TokenLocal {
            id: "9".to_string(),
            count: 0,
            bracket: 6,

            level: "Special".to_string(),
        },
        TokenLocal {
            id: "0".to_string(),
            count: 0,
            bracket: 6,

            level: "Special".to_string(),
        },
        TokenLocal {
            id: "18349153976137682097687065310984821295737582987254388036615603420290411921433"
                .to_string(),
            count: 0,
            bracket: 7,

            level: "Rare".to_string(),
        },
        TokenLocal {
            id: "11".to_string(),
            count: 0,
            bracket: 7,

            level: "Rare".to_string(),
        },
        TokenLocal {
            id: "18349153976137682097687065310984821295737582987254388036615603448877714243609"
                .to_string(),
            count: 0,
            bracket: 8,

            level: "Rare".to_string(),
        },
        TokenLocal {
            id: "12".to_string(),
            count: 0,
            bracket: 8,

            level: "Rare".to_string(),
        },
        TokenLocal {
            id: "18349153976137682097687065310984821295737582987254388036615603446678690988057"
                .to_string(),
            count: 0,
            bracket: 9,

            level: "Rare".to_string(),
        },
        TokenLocal {
            id: "18349153976137682097687065310984821295737582987254388036615603449977225871385"
                .to_string(),
            count: 0,
            bracket: 9,

            level: "Rare".to_string(),
        },
        TokenLocal {
            id: "18349153976137682097687065310984821295737582987254388036615603447778202615833"
                .to_string(),
            count: 0,
            bracket: 10,

            level: "Rare".to_string(),
        },
        TokenLocal {
            id: "13".to_string(),
            count: 0,
            bracket: 10,

            level: "Rare".to_string(),
        },
        TokenLocal {
            id: "14".to_string(),
            count: 0,
            bracket: 11,

            level: "Rare".to_string(),
        },
        TokenLocal {
            id: "18349153976137682097687065310984821295737582987254388036615603435683574710297"
                .to_string(),
            count: 0,
            bracket: 11,

            level: "Rare".to_string(),
        },
    ];
    for token in &result {
        let new_token = NewToken {
            id: &token.id,
            count: &token.count,
            bracket: &token.bracket,
            level: &token.level,
        };
        let connection = &mut establish_connection();

        diesel::insert_into(tokens::table())
            .values(new_token)
            .returning(Token::as_returning())
            .get_result(connection)
            .expect("Error saving new post");
    }
    HttpResponse::Ok().json("Oke")
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
