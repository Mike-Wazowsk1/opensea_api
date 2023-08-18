use self::schema::info::dsl::*;
use self::schema::tokens::dsl::*;

use actix_web::web;
use diesel::pg::PgConnection;
use diesel::prelude::*;
use dotenvy::dotenv;
use ethers::prelude::*;
use ethers::prelude::rand::seq::SliceRandom;
use ethers::providers::{Http, Provider};
use moka::sync::Cache;
use opensea_api::models::{InfoPoint, Token};
use opensea_api::*;
use std::str::FromStr;
use std::sync::Arc;
use std::time::Duration;

use crate::structs;
use rand::Rng;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::{collections::HashMap, error::Error};
use std::{env, thread};
use tokio::task;

pub const MATICURL: &str = "https://polygon-rpc.com";

abigen!(
    NftContract,
    "abi.json",
    event_derives(serde::Deserialize, serde::Serialize)
);

pub async fn get_collection_from_opensea() -> Result<structs::NFTResponse, Box<dyn Error>> {
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
pub async fn get_counts_local(
    client: &Provider<Http>,
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

pub async fn establish_connection() -> PgConnection {
    dotenv().ok();
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    PgConnection::establish(&database_url)
        .unwrap_or_else(|_| panic!("Error connecting to {}", database_url))
}

pub async fn make_nft_array(connection: &mut PgConnection) -> Vec<structs::TokenLocal> {
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

pub async fn get_counts(
    client: &Provider<Http>,
    contract_addr: &H160,
    address: &web::Path<String>,
    nfts: &mut Vec<structs::TokenLocal>,
) -> Vec<U256> {
    let contract: NftContract<&Provider<Http>> =
        NftContract::new(contract_addr.clone(), Arc::new(&client));

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

pub async fn multiplicator(tokens_arr: &Vec<structs::TokenLocal>) -> Vec<f64> {
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

pub async fn get_pts(tokens_arr: &Vec<structs::TokenLocal>) -> f64 {
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
pub async fn get_pts_by_grade(tokens_arr: &Vec<structs::TokenLocal>) -> HashMap<String, f64> {
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

//
pub async fn get_nft_by_address_local(
    nfts: &mut Vec<structs::TokenLocal>,
    address: &String,
    client: &Provider<Http>,
    contract_addr: &H160,
) -> f64 {
    let _balance = get_counts_local(&client, &contract_addr, &address, nfts).await;
    let pts: f64 = get_pts(&nfts).await;
    pts
}

pub async fn get_ids(connection: &mut PgConnection) -> (Vec<String>, Vec<structs::TokenLocal>) {
    let mut token_ids = Vec::new();
    // let connection = &mut establish_connection().await;
    let nfts: Vec<structs::TokenLocal> = make_nft_array(connection).await;

    for n in &nfts {
        token_ids.push((*n.id).to_string());
    }

    (token_ids, nfts)
}

pub async fn get_owners_local(cache: Cache<String, f64>) {
    let mut connection: &mut PgConnection = &mut establish_connection().await;
    let contract_addr = Address::from_str("0x2953399124F0cBB46d2CbACD8A89cF0599974963").unwrap();

    // let mut scores: HashMap<String, f64> = HashMap::new();

    let provider = Provider::<Http>::try_from(MATICURL).unwrap();
    let tup = get_ids(&mut connection).await;
    let nfts: Vec<structs::TokenLocal> = tup.1;

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

            for owner in tmp_owners.owners {
                let ok_owner: String = match owner {
                    Some(x) => x,
                    None => continue,
                };

                let mut tasks = Vec::new();

                // let nfts = &nfts;

                // let ok_owner = owner.clone();
                let client = provider.clone();
                let mut nfts = nfts.clone();
                // let new_ok_owner = ok_owner.clone();

                let task = task::spawn(async move {
                    let current_address =
                        get_nft_by_address_local(&mut nfts, &ok_owner, &client, &contract_addr)
                            .await;
                    let current_pts = current_address;
                    (ok_owner, current_pts)
                });
                tasks.push(task);

                for task in tasks {
                    let (tmp_owner, current_pts) = task.await.unwrap();
                    let cache_value = cache.get(&tmp_owner);
                    match cache_value {
                        Some(value) => {
                            if current_pts != value {
                                cache.insert(tmp_owner, value);
                            }
                        }
                        None => {
                            cache.insert(tmp_owner, current_pts);
                        }
                    };
                    // scores.insert(tmp_owner, current_pts);
                }
                // if !cache.contains_key(&new_ok_owner) {
                //     cache.insert(new_ok_owner, 99.);
                // }
            }
        }
        thread::sleep(Duration::from_millis(300000));
    }
}

pub async fn wbgl(connection: &mut PgConnection) -> f64 {
    let value = info.load::<InfoPoint>(connection).unwrap();
    value[0].wbgl.unwrap() as f64
}

pub async fn get_ticket_count(sum_wbgl: f64) -> i32 {
    if sum_wbgl >= 10_000. {
        return 100_000;
    }
    if sum_wbgl >= 1_000. {
        return 10_000;
    }
    1_000
}

pub async fn get_ticket_weight(sum_wbgl: f64) -> f64 {
    if sum_wbgl >= 10_000. && sum_wbgl < 75_000. {
        return 75_000. / sum_wbgl;
    }
    if sum_wbgl >= 1_000. && sum_wbgl < 7_500. {
        return 7_500. / sum_wbgl;
    }
    if sum_wbgl < 750. {
        return 750. / sum_wbgl;
    }
    1.
}
pub async fn get_ticket_array(ticket_count: i32) -> Vec<i32> {
    vec![-1; ticket_count as usize]
}

fn calculate_hash<T: Hash>(t: &T) -> u64 {
    let mut s = DefaultHasher::new();
    t.hash(&mut s);
    s.finish()
}

pub async fn generate_sequence(data: f64,size:i32) ->Vec<i32>{
    let state = calculate_hash(&(data as i32));
    let mut r = <rand::rngs::StdRng as rand::SeedableRng>::seed_from_u64(state);
    let mut sequence = vec![];

    for i in 0..size {
        sequence.push(i);
    }
    sequence.shuffle(&mut r);
    sequence
}
