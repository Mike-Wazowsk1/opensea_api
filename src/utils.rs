use crate::schema::tokens::dsl::*;

use crate::models::Token;
use crate::schema::info_lotto::dsl::*;
use actix_web::web;
use diesel::pg::PgConnection;
use diesel::r2d2::ConnectionManager;
use diesel::{prelude::*, r2d2};
use dotenvy::dotenv;
use ethers::prelude::rand::seq::SliceRandom;
use ethers::prelude::*;
use ethers::providers::{Http, Provider};
use moka::sync::Cache;
// use crate::*;
use std::io::{BufWriter, Write};
use std::path::Path;
// use random_color::RandomColor;
// use serde_json::json;
use std::str::FromStr;
use std::sync::Arc;
use std::thread;
use std::time::Duration;
// use std::time::Duration;

use crate::models::InfoLottoPoint;
use std::collections::HashMap;

use crate::structs;
use std::collections::hash_map::DefaultHasher;
use std::env;
use std::error::Error;
use std::hash::{Hash, Hasher};
use std::process::Command;
use tokio::task;

pub const MATICURL: &str = "https://polygon-rpc.com";
pub const CONTRACT_ADDRESS: &str = "0xd74d5fe12ebc67075d18a74e2da9a06334c7335e";
pub const OWNER_ADDRESS: &str = "0x4c1c5403e419d736f267bbac8911454bd0ba9043";
abigen!(
    NftContract,
    "abi.json",
    event_derives(serde::Deserialize, serde::Serialize)
);

pub async fn get_collection_from_opensea() -> Result<structs::NFTResponse, Box<dyn Error>> {
    let client = reqwest::Client::builder().build()?;

    let resp = client
        .get("https://api.opensea.io/v2/collection/new-bitgesell-road/nfts?limit=50")
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
    let mut db: Vec<Token> = match tokens.load(connection) {
        Ok(x) => x,
        Err(err) => {
            println!("Get tokens info error: {:?}", err);
            return result;
        }
    };
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
    let nfts: Vec<structs::TokenLocal> = make_nft_array(connection).await;

    for n in &nfts {
        token_ids.push((*n.id).to_string());
    }

    (token_ids, nfts)
}

pub async fn get_winning_block(connection: &mut PgConnection) -> u128 {
    let value = info_lotto.load::<InfoLottoPoint>(connection).unwrap();
    let s = value[0].wining_block.clone().unwrap();

    s as u128
}

pub async fn get_owners_local(cache: Arc<Cache<String, f64>>) {
    let mut connection: &mut PgConnection = &mut establish_connection().await;
    let tmp = CONTRACT_ADDRESS.clone().to_string();
    let own = OWNER_ADDRESS.clone().to_string();
    let contract_addr = Address::from_str(&tmp).unwrap();
    let provider = Provider::<Http>::try_from(MATICURL).unwrap();

    loop {
        let mut owners_real = vec![];
        let tup = get_ids(&mut connection).await;
        let nfts: Vec<structs::TokenLocal> = tup.1;
        let token_ids = tup.0;

        for tok in token_ids {
            if tok == "NO_VALUE".to_string() {
                continue;
            }

            let url = format!(
                "https://polygon-mainnet.g.alchemy.com/nft/v2/lUgTmkM2_xJvUIF0dB1iFt0IQrqd4Haw/getOwnersForToken?contractAddress={tmp}&tokenId={tok}",
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
                Err(x) => {
                    println!("Err tmp_serde: {:?}", x);
                    structs::OwnersResponse {
                        owners: Vec::new(),
                        page_key: Option::None,
                    }
                }
            };
            for owner in tmp_owners.owners {
                let ok_owner: String = match owner {
                    Some(x) => x,
                    None => continue,
                };
                if ok_owner == own {
                    continue;
                }

                let mut tasks = Vec::new();

                let client = provider.clone();
                let mut nfts = nfts.clone();
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
                    owners_real.push(tmp_owner.clone());
                    let cache_value = cache.get(&tmp_owner);
                    match cache_value {
                        Some(value) => {
                            if current_pts != value {
                                cache.insert(tmp_owner, current_pts);
                            }
                        }
                        None => {
                            cache.insert(tmp_owner, current_pts);
                        }
                    };
                }
            }
        }
        let stored: Vec<(Arc<String>, f64)> = cache.iter().collect();
        let mut stored_owners: Vec<String> = vec![];
        for (s, _f) in stored {
            stored_owners.push(s.to_string())
        }
        let missing_owners: Vec<&String> = stored_owners
            .iter()
            .filter(|owner| !owners_real.contains(owner))
            .collect();
        for missing_owner in missing_owners {
            if missing_owner == "last_lucky_block" || missing_owner == "last_lucky_wbgl" {
                continue;
            }
            cache.remove(missing_owner);
        }
        let current_block = get_current_block().await;
        let lucky_block = get_winning_block(&mut connection).await;
        let mut owners_map: Vec<(String, f64)> = vec![];
        let owners_map_t: Vec<(Arc<String>, f64)> = cache.iter().collect();
        for (k, v) in owners_map_t {
            if *k == "last_lucky_block" || *k == "last_lucky_wbgl" {
                continue;
            }
            let key = k.to_string();
            owners_map.push((key, v));
        }
        let mut sum_wbgl = 0.;
        for st in &owners_map {
            sum_wbgl += st.1;
        }
        println!("{:?}", owners_map);
        if current_block > lucky_block {
            cache.insert("last_lucky_block".to_string(), lucky_block as f64);
            cache.insert("last_lucky_wbgl".to_string(), sum_wbgl as f64);
            let dir = env::current_dir().unwrap();
            let filename = format!("/{lucky_block}.json");

            let dir = dir.into_os_string().into_string().unwrap() + "/snapshots" + &filename;
            // println!("{:?}", dir);
            if !Path::new(&dir).exists() {
                let file = match std::fs::File::create(dir) {
                    Ok(x) => x,
                    Err(x) => {
                        println!("createFileError: {:?}", x);
                        continue;
                    }
                };
                let mut writer = BufWriter::new(file);
                match serde_json::to_writer(&mut writer, &owners_map) {
                    Ok(x) => x,
                    Err(x) => {
                        println!("SerdeToWritterError: {:?}", x);
                        continue;
                    }
                };
                match writer.flush() {
                    Ok(x) => x,
                    Err(x) => {
                        println!("FlushError: {:?}", x);
                        continue;
                    }
                };
            }
        }
        thread::sleep(Duration::from_millis(30000));
    }
}

pub async fn wbgl(connection: &mut PgConnection) -> f64 {
    let value = info_lotto.load::<InfoLottoPoint>(connection).unwrap();
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
    if sum_wbgl >= 10_000. && sum_wbgl < 70_000. {
        return 70_000. / sum_wbgl;
    }
    if sum_wbgl >= 1_000. && sum_wbgl < 7_000. {
        return 7_000. / sum_wbgl;
    }
    if sum_wbgl < 700. {
        return 700. / sum_wbgl;
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

pub async fn generate_sequence(data: f64, current_block: u128, size: i32) -> Vec<i32> {
    let w = get_ticket_weight(data).await;
    let mut data_clone = data.clone();
    if w == 1. {
        if data > 1000. && data < 10_000. {
            data_clone = 7_000.
        }
        if data > 10_000. && data < 100_000. {
            data_clone = 70_000.
        }
        if data < 1000. {
            data_clone = 700.
        }
    }
    let key = format!("{data_clone}{current_block}");

    let state = calculate_hash(&key);
    let mut r = <rand::rngs::StdRng as rand::SeedableRng>::seed_from_u64(state);
    let mut sequence = vec![];

    for i in 0..size {
        sequence.push(i);
    }
    sequence.shuffle(&mut r);
    sequence
}

pub async fn get_current_block() -> u128 {
    let out = match Command::new("BGL-cli").arg("getblockcount").output() {
        Ok(x) => x,
        Err(_) => return 0 as u128,
    };

    let str_block = String::from_utf8_lossy(&out.stdout);
    let mut s = str_block.to_string();
    s.pop();
    let block: u128 = match s.parse() {
        Ok(x) => x,
        Err(_) => return 0 as u128,
    };
    block
}
pub async fn get_block_hash(block: u128) -> String {
    let arg = format!("{}", block);
    let a = "getblockhash";
    let out = Command::new("BGL-cli")
        .arg(a)
        .arg(arg)
        .output()
        .expect("ls command failed to start");
    let str_block = String::from_utf8_lossy(&out.stdout);
    let mut s = str_block.to_string();
    s.pop();
    s
}
pub async fn get_lucky_block(
    mut connection: r2d2::PooledConnection<ConnectionManager<PgConnection>>,
) -> u128 {
    let value = info_lotto.load::<InfoLottoPoint>(&mut connection).unwrap();
    let s = value[0].wining_block.clone().unwrap();

    s as u128
}
pub async fn get_round(
    mut connection: r2d2::PooledConnection<ConnectionManager<PgConnection>>,
) -> i32 {
    let value = info_lotto.load::<InfoLottoPoint>(&mut connection).unwrap();
    value[0].round.unwrap()
}

pub async fn get_minted_tickets(
    sum_wbgl: f64,
    current_block: u128,
    owners_map: &mut Vec<(Arc<String>, f64)>,
) -> (Vec<i32>, HashMap<i32, structs::TicketInfo>) {
    let mut i = 0;
    let mut j = 0;
    let mut colors: HashMap<i32, structs::TicketInfo> =
        HashMap::with_capacity(owners_map.capacity());

    owners_map.sort_by(|a, b| {
        let score_comparison = b.1.partial_cmp(&a.1).unwrap();
        if score_comparison == std::cmp::Ordering::Equal {
            (*a.0).partial_cmp(&(*b.0)).unwrap()
        } else {
            score_comparison
        }
    });

    let ticket_weight = get_ticket_weight(sum_wbgl).await;
    let ticket_count = get_ticket_count(sum_wbgl).await;

    let mut tickets = get_ticket_array(ticket_count).await;
    let sequence = generate_sequence(sum_wbgl, current_block, ticket_count).await;

    owners_map.iter().for_each(|(address, score)| {
        // let color = RandomColor::new().to_hex();
        colors.insert(
            i,
            structs::TicketInfo {
                address: address.to_string(),
                // color,
            },
        );
        let tickets_for_user = (ticket_weight * score) as i32;
        for _j in 0..tickets_for_user {
            if (j as usize) < sequence.len() {
                if (sequence[j as usize] as usize) < tickets.len() {
                    tickets[sequence[j as usize] as usize] = i;
                    j += 1;
                }
            }
        }

        i += 1;
    });
    (tickets, colors)
}

fn get_winners(vec: Vec<i32>, n: usize) -> Vec<i32> {
    let mut groups: Vec<Vec<i32>> = Vec::new();
    let mut res = vec![];

    let mut i: i32 = vec.len() as i32;
    while i > 0 {
        let start = if i as usize >= n { i as usize - n } else { 0 };
        let group = vec[start..i as usize].to_vec();
        groups.push(group);
        i -= n as i32;
    }

    groups.reverse();

    for group in groups {
        let vec_i32: Vec<i32> = group.iter().map(|&x| x as i32).collect();

        let number: i32 = vec_i32.iter().fold(0, |acc, &x| acc * 10 + x);
        res.push(number);
    }
    res.reverse();
    res
}
fn parse_digits(t_num: &str) -> Vec<i32> {
    let group: Vec<u32> = t_num.chars().filter_map(|a| a.to_digit(10)).collect();
    let t_num: Vec<i32> = group.iter().map(|&x| x as i32).collect();
    t_num
}

pub async fn get_win_tickets(h: String, l: i32) -> Vec<i32> {
    if l == 1000 {
        let h = parse_digits(&h);
        let winners: Vec<i32> = get_winners(h, 3);
        if winners.len() == 0 {
            return vec![-1000, -1000, -1000];
        }
        return winners[0..3].to_vec();
    }
    if l == 10_000 {
        let h = parse_digits(&h);
        let winners = get_winners(h, 4);
        if winners.len() == 0 {
            return vec![-1000, -1000, -1000];
        }
        return winners[0..4].to_vec();
    }
    if l == 100_000 {
        let h = parse_digits(&h);
        let winners = get_winners(h, 5);
        if winners.len() == 0 {
            return vec![-1000, -1000, -1000];
        }
        return winners[0..5].to_vec();
    }
    return Vec::new();
}
