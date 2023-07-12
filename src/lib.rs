pub mod models;
pub mod schema;

#[cfg(test)]
mod tests {
    #[derive(Debug, Clone)]

    pub struct TokenLocal {
        index: i32,
        count: i32,
        id: String,
        bracket: i32,
        level: String,
    }

    pub struct TokenLocalTmp {
        index: i32,
        count: i32,
        id: String,
        bracket: i32,
        level: String,
        is_full: bool,
    }


    // #[test]
    // fn test_get_pts() {
    //     let tokens_arr = vec![
    //         TokenLocal {
    //             level: "Common".to_string(),
    //             bracket: 0,
    //             count: 2,
    //         },
    //         TokenLocal {
    //             level: "Rare".to_string(),
    //             bracket: 1,
    //             count: 1,
    //         },
    //     ];

    //     let mut rt = Runtime::new().unwrap();
    //     let result = rt.block_on(get_pts(&tokens_arr));
    //     assert_eq!(result, 10.0);
    // }

    // #[test]
    // fn test_get_pts_by_grade() {
    //     let tokens_arr = vec![
    //         TokenLocal {
    //             level: "Common".to_string(),
    //             bracket: 0,
    //             count: 2,
    //         },
    //         TokenLocal {
    //             level: "Rare".to_string(),
    //             bracket: 1,
    //             count: 1,
    //         },
    //         TokenLocal {
    //             level: "Legendary".to_string(),
    //             bracket: 2,
    //             count: 3,
    //         },
    //     ];

    //     let mut rt = Runtime::new().unwrap();
    //     let result = rt.block_on(get_pts_by_grade(&tokens_arr));

    //     let mut expected_scores = HashMap::new();
    //     expected_scores.insert("Common".to_string(), 2.0);
    //     expected_scores.insert("Special".to_string(), 0.0);
    //     expected_scores.insert("Rare".to_string(), 7.0);
    //     expected_scores.insert("Unique".to_string(), 0.0);
    //     expected_scores.insert("Legendary".to_string(), 150.0);

    //     assert_eq!(result, expected_scores);
    // }

    #[test]
    fn test_is_full() {
        let tokens_arr = vec![
            TokenLocal {
                index: 0,
                id: "18349153976137682097687065310984821295737582987254388036615603441181132849302"
                    .to_string(),
                count: 1,
                bracket: 0,
                level: "Common".to_string(),
            },
            TokenLocal {
                index: 1,
                id: "18349153976137682097687065310984821295737582987254388036615603429086504943816"
                    .to_string(),
                count: 1,
                bracket: 0,
                level: "Common".to_string(),
            },
            TokenLocal {
                index: 2,
                id: "18349153976137682097687065310984821295737582987254388036615603443380156104854"
                    .to_string(),
                count: 89,
                bracket: 0,
                level: "Common".to_string(),
            },
            TokenLocal {
                index: 3,
                id: "18349153976137682097687065310984821295737582987254388036615603437882597965974"
                    .to_string(),
                count: 889,
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
                count: 2,
                bracket: 1,
                level: "Common".to_string(),
            },
            TokenLocal {
                index: 6,
                id: "NO_VALUE".to_string(),
                count: 1,
                bracket: 1,
                level: "Common".to_string(),
            },
            TokenLocal {
                index: 7,
                id: "18349153976137682097687065310984821295737582987254388036615603418091388666006"
                    .to_string(),
                count: 4,
                bracket: 1,
                level: "Common".to_string(),
            },
            TokenLocal {
                index: 8,
                id: "18349153976137682097687065310984821295737582987254388036615603451076737499211"
                    .to_string(),
                count: 1,
                bracket: 2,
                level: "Special".to_string(),
            },
            TokenLocal {
                index: 9,
                id: "18349153976137682097687065310984821295737582987254388036615603432385039827019"
                    .to_string(),
                count: 1,
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
                count: 1,
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
                count: 1,
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
                count: 1,
                bracket: 6,
                level: "Special".to_string(),
            },
            TokenLocal {
                index: 21,
                id: "NO_VALUE".to_string(),
                count: 1,
                bracket: 6,
                level: "Special".to_string(),
            },
            TokenLocal {
                index: 22,
                id: "NO_VALUE".to_string(),
                count: 1,
                bracket: 6,
                level: "Special".to_string(),
            },
        ];
        let mut res: Vec<TokenLocalTmp> = Vec::new();
        for token_local in  &tokens_arr{
            let bracket = token_local.bracket;
            let is_full = tokens_arr.iter().filter(|&t| t.bracket == bracket)
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
    
        println!("{:?}", res);
    }
}
