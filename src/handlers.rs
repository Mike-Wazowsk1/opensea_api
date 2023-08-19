use self::schema::info::dsl::*;
use self::schema::tokens::dsl::*;
use random_color::{Color, Luminosity, RandomColor};

use actix_web::{get, web, HttpRequest, HttpResponse, Responder};
use diesel::associations::HasTable;
use diesel::pg::PgConnection;
use diesel::prelude::*;
use diesel::r2d2::{self, ConnectionManager};
use ethers::prelude::*;
use ethers::providers::{Http, Provider};
use moka::sync::Cache;
use opensea_api::models::{InfoPoint, NewToken, Token};
use opensea_api::*;
use std::collections::HashMap;
use std::ops::Add;
use std::str::FromStr;

use std::sync::Arc;

use crate::structs;
use crate::utils;

#[get("/info")]
pub async fn get_nfts() -> impl Responder {
    match utils::get_collection_from_opensea().await {
        Ok(nfts) => HttpResponse::Ok()
            .append_header(("Access-Control-Allow-Origin", "*"))
            .json(nfts),
        Err(err) => HttpResponse::InternalServerError().body(err.to_string()),
    }
}

#[get("/get_blockchain_data")]
pub async fn get_blockchain_data(
    pool: web::Data<r2d2::Pool<ConnectionManager<PgConnection>>>,
) -> impl Responder {
    let last_block = utils::get_current_block().await;
    let lucky_block = utils::get_lucky_block().await;
    let mut blocks_before = 0;
    if last_block.le(&lucky_block) {
        blocks_before = lucky_block - last_block;
    }

    HttpResponse::Ok()
        .append_header(("Access-Control-Allow-Origin", "*"))
        .json(structs::BlockChainData {
            winning_block: lucky_block,
            blocks_before: blocks_before,
        })
}
#[get("/get_last_winners")]
pub async fn get_last_winners() -> impl Responder {
    match utils::get_collection_from_opensea().await {
        Ok(nfts) => HttpResponse::Ok()
            .append_header(("Access-Control-Allow-Origin", "*"))
            .json(nfts),
        Err(err) => HttpResponse::InternalServerError().body(err.to_string()),
    }
}
#[get("/get_lucky_hash")]
pub async fn get_lucky_hash() -> impl Responder {
    let last_block = utils::get_current_block().await;
    let lucky_block = utils::get_lucky_block().await;
    if last_block >= lucky_block {
        let lucky_hash = utils::get_block_hash(lucky_block).await;
        return HttpResponse::Ok()
            .append_header(("Access-Control-Allow-Origin", "*"))
            .json(lucky_hash);
    }
    HttpResponse::Ok()
        .append_header(("Access-Control-Allow-Origin", "*"))
        .json(Address::zero())
}

#[get("/get_tickets_count")]
pub async fn get_tickets_count(
    pool: web::Data<r2d2::Pool<ConnectionManager<PgConnection>>>,
    cache: web::Data<Cache<String, f64>>,
) -> impl Responder {
    let mut sorted_scores: Vec<(Arc<String>, f64)> = cache.iter().collect();
    sorted_scores.sort_by(|a, b| {
        let score_comparison = b.1.partial_cmp(&a.1).unwrap();
        if score_comparison == std::cmp::Ordering::Equal {
            (*a.0).partial_cmp(&(*b.0)).unwrap()
        } else {
            score_comparison
        }
    });
    let mut sum_wbgl = 0.;
    for st in &sorted_scores {
        sum_wbgl += st.1;
    }

    let ticket_count = utils::get_ticket_count(sum_wbgl).await;

    HttpResponse::Ok()
        .append_header(("Access-Control-Allow-Origin", "*"))
        .json(ticket_count)
}

#[get("/get_tickets")]
pub async fn get_tickets(
    pool: web::Data<r2d2::Pool<ConnectionManager<PgConnection>>>,
    cache: web::Data<Cache<String, f64>>,
) -> impl Responder {
    let mut sorted_scores: Vec<(Arc<String>, f64)> = cache.iter().collect();
    let mut connection = pool.get().unwrap();
    let mut colors = HashMap::with_capacity(sorted_scores.capacity());
    let mut i = 0;
    let mut j = 0;

    let wbgl_points = utils::wbgl(&mut connection).await;

    sorted_scores.sort_by(|a, b| {
        let score_comparison = b.1.partial_cmp(&a.1).unwrap();
        if score_comparison == std::cmp::Ordering::Equal {
            (*a.0).partial_cmp(&(*b.0)).unwrap()
        } else {
            score_comparison
        }
    });
    let mut sum_wbgl = 0.;
    for st in &sorted_scores {
        sum_wbgl += st.1;
    }

    let mut ticket_weight = utils::get_ticket_weight(sum_wbgl).await;
    let mut ticket_count = utils::get_ticket_count(sum_wbgl).await;

    let mut tickets = utils::get_ticket_array(ticket_count).await;
    let sequence = utils::generate_sequence(sum_wbgl, ticket_count).await;

    sorted_scores.iter().for_each(|(address, score)| {
        let color = RandomColor::new()
            // .hue(Color::Blue) // Optional
            // .luminosity(Luminosity::Light) // Optional
            // .seed(42) // Optional
            // .alpha(1.0) // Optional
            .to_hex();
        colors.insert(
            i,
            structs::TicketInfo {
                address: address.to_string(),
                color: color,
            },
        );
        let tickets_for_user = (ticket_weight * score) as i32;
        for _j in 0..tickets_for_user {
            tickets[sequence[j as usize] as usize] = i;
            j += 1;
        }

        i += 1;
    });
    let resp = structs::TicketResponse {
        tickets,
        map: colors,
    };

    HttpResponse::Ok()
        .append_header(("Access-Control-Allow-Origin", "*"))
        .json(resp)
}

#[get("/get_owners")]
pub async fn get_owners(
    req: HttpRequest,
    pool: web::Data<r2d2::Pool<ConnectionManager<PgConnection>>>,
    cache: web::Data<Cache<String, f64>>,
) -> impl Responder {
    let q: String = req.query_string().replace("&", " ").replace("=", " ");
    let query: Vec<&str> = q.split(" ").collect();
    let mut connection = pool.get().unwrap();

    // let contract_addr = Address::from_str("0x2953399124F0cBB46d2CbACD8A89cF0599974963").unwrap();

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
    // let mut scores: HashMap<String, f64> = HashMap::new();

    // unsafe {
    //     let mut tasks = Vec::new();
    //     let provider = Provider::<Http>::try_from(MATICURL).unwrap();

    //     // let nfts = &nfts;
    //     let tup = get_ids(&mut connection).await;
    //     let nfts: Vec<structs::TokenLocal> = tup.1;

    // for owner in GLOBAL_OWNERS.iter() {
    //     let ok_owner = owner.clone();
    //     if !scores.contains_key(&ok_owner) {
    //         let client = provider.clone();
    //         let mut nfts = nfts.clone();

    //         let task = task::spawn(async move {
    //             let current_address =
    //                 get_nft_by_address_local(&mut nfts, &ok_owner, &client, &contract_addr)
    //                     .await;
    //             let current_pts = current_address;
    //             (ok_owner, current_pts)
    //         });
    //         tasks.push(task);
    //     }
    // }

    //     for task in tasks {
    //         let (ok_owner, current_pts) = task.await.unwrap();
    //         scores.insert(ok_owner, current_pts);
    //     }
    // }

    let mut sorted_scores: Vec<(Arc<String>, f64)> = cache.iter().collect();

    sorted_scores.sort_by(|a, b| {
        let score_comparison = b.1.partial_cmp(&a.1).unwrap();
        if score_comparison == std::cmp::Ordering::Equal {
            (*a.0).partial_cmp(&(*b.0)).unwrap()
        } else {
            score_comparison
        }
    });
    let mut sum_wbgl = 0.;
    for st in &sorted_scores {
        sum_wbgl += st.1;
    }

    let wbgl_points = utils::get_ticket_weight(sum_wbgl).await;

    let mut result = Vec::new();

    for i in 0..sorted_scores.len() {
        let reward = (wbgl_points * sorted_scores[i].1);
        if search == "" {
            result.push(structs::Fun2Response {
                address: sorted_scores[i].0.to_string(),
                score: sorted_scores[i].1,
                reward: reward as i64,
            });
        } else {
            if sorted_scores[i].0.to_string() == search {
                result.push(structs::Fun2Response {
                    address: sorted_scores[i].0.to_string(),
                    score: sorted_scores[i].1,
                    reward: reward as i64,
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
        let reward = (wbgl_points * sorted_scores[i].1);
        final_result.push(structs::Fun2Response {
            address: sorted_scores[i].0.to_string(),
            score: sorted_scores[i].1,
            reward: reward as i64,
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

#[get("/get_last_trade")]
pub async fn get_last_trade(
    pool: web::Data<r2d2::Pool<ConnectionManager<PgConnection>>>,
) -> impl Responder {
    let mut connection = pool.get().unwrap();
    let value = info.load::<InfoPoint>(&mut connection).unwrap();
    let last_tx = value[0].hash.clone();

    let href = format!("https://bscscan.com/tx/{last_tx}");

    let response = structs::LastTradeResponse {
        hash: last_tx,
        href,
    };
    HttpResponse::Ok()
        .append_header(("Access-Control-Allow-Origin", "*"))
        .json(response)
}

#[get("/get_pages/{limit}")]
pub async fn get_pages(
    limit: web::Path<i32>,
    cache: web::Data<Cache<String, f64>>,
) -> impl Responder {
    let z: i32;

    let a = cache.entry_count() as i32;
    let b = limit.into_inner();
    if a % b == 0 {
        z = a / b;
    } else {
        z = a / b + 1;
    }
    HttpResponse::Ok()
        .append_header(("Access-Control-Allow-Origin", "*"))
        .json(z)
}

#[get("/get_payment")]
pub async fn get_payment(
    pool: web::Data<r2d2::Pool<ConnectionManager<PgConnection>>>,
    cache: web::Data<Cache<String, f64>>,
) -> impl Responder {
    let mut connection = pool.get().unwrap();

    // let contract_addr = Address::from_str("0x2953399124F0cBB46d2CbACD8A89cF0599974963").unwrap();

    // let mut scores: HashMap<String, f64> = HashMap::new();

    // let provider = Provider::<Http>::try_from(utils::MATICURL).unwrap();
    // let tup = utils::get_ids(&mut connection).await;
    // let nfts: Vec<structs::TokenLocal> = tup.1;
    // unsafe {
    //     let mut tasks = Vec::new();

    //     for owner in GLOBAL_OWNERS.iter() {
    //         let ok_owner = owner.clone();
    //         if !scores.contains_key(&ok_owner) {
    //             let client = provider.clone();
    //             let mut nfts = nfts.clone();

    //             let task = task::spawn(async move {
    //                 let current_address =
    //                     utils::get_nft_by_address_local(&mut nfts, &ok_owner, &client, &contract_addr)
    //                         .await;
    //                 let current_pts = current_address;
    //                 (ok_owner, current_pts)
    //             });
    //             tasks.push(task);
    //         }
    //     }

    //     for task in tasks {
    //         let (ok_owner, current_pts) = task.await.unwrap();
    //         scores.insert(ok_owner, current_pts);
    //     }
    // }

    let mut sorted_scores: Vec<(Arc<String>, f64)> = cache.iter().collect();
    let wgbl_score = utils::wbgl(&mut connection).await;

    sorted_scores.sort_by(|a, b| {
        let score_comparison = b.1.partial_cmp(&a.1).unwrap();
        if score_comparison == std::cmp::Ordering::Equal {
            a.0.partial_cmp(&b.0).unwrap()
        } else {
            score_comparison
        }
    });

    let mut sum_wbgl = 0.;
    for st in &sorted_scores {
        sum_wbgl += st.1;
    }
    let mut result: Vec<String> = Vec::new();
    for i in 0..sorted_scores.len() {
        let reward = (wgbl_score * sorted_scores[i].1) / sum_wbgl;

        let str_reward = format!("{}", reward);
        result.push(format!("{}?{}", sorted_scores[i].0, str_reward));
    }
    let text = result.join(";");

    HttpResponse::Ok()
        .append_header(("Access-Control-Allow-Origin", "*"))
        .json(text)
}

#[get("/nft/{address}")]
pub async fn get_nft_by_address(
    address: web::Path<String>,
    pool: web::Data<r2d2::Pool<ConnectionManager<PgConnection>>>,
) -> impl Responder {
    // let connection = &mut establish_connection().await;
    let mut connection = pool.get().unwrap();

    let provider: Provider<Http> = Provider::<Http>::try_from(utils::MATICURL).unwrap();
    let mut nfts: Vec<structs::TokenLocal> = utils::make_nft_array(&mut connection).await;

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

    let _balance = utils::get_counts(&provider, &contract_addr, &address, &mut nfts).await;

    let sum_pts = utils::get_pts(&nfts).await;
    let pts_by_grade = utils::get_pts_by_grade(&nfts).await;
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

#[get("/get_wbgl")]
pub async fn get_wbgl(
    pool: web::Data<r2d2::Pool<ConnectionManager<PgConnection>>>,
) -> impl Responder {
    let mut conn = pool.get().unwrap();
    // let connection: &mut PgConnection = &mut establish_connection().await;

    // let wbgl

    HttpResponse::Ok()
        .append_header(("Access-Control-Allow-Origin", "*"))
        .json(utils::wbgl(&mut conn).await)
}

#[get("/init_db")]
pub async fn init_db(
    pool: web::Data<r2d2::Pool<ConnectionManager<PgConnection>>>,
) -> impl Responder {
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
