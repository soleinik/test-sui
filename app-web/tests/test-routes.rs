use std::collections::BTreeSet;

use app_data::{self, BalanceChange};
use app_web;
use axum::http::StatusCode;
use axum_test_helpers::TestClient;
mod data_creator;
use data_creator::*;

#[tokio::test]
async fn my_test() {
    let router = app_web::router();
    let client = TestClient::new(router);
    let orig = get_balance_rand(99999);
    let orig_01 = get_balance_rand(8888);

    //post list of 1
    let res = client
        .post(app_web::PATH_POST_BALANCE)
        .json(&vec![orig.clone(), orig_01.clone()])
        .await;

    assert!(res.status() == StatusCode::CREATED);

    //get one
    let res = client
        .get(&app_web::PATH_GET_BALANCE_ACCOUNT.replace(":account_id", &orig.address))
        .await;

    assert!(res.status() == StatusCode::OK);
    let actual = res.json::<BalanceChange>().await;
    assert_eq!(orig, actual);

    //negative
    let res = client
        .get(&app_web::PATH_GET_BALANCE_ACCOUNT.replace(":account_id", "AAA-BBB-CCC"))
        .await;

    assert!(res.status() == StatusCode::NOT_FOUND);

    //get all
    let res = client.get(&app_web::PATH_GET_BALANCE_LIST).await;
    assert!(res.status() == StatusCode::OK);
    let actual = res.json::<BTreeSet<BalanceChange>>().await;
    assert_eq!(actual.len(), 2);
    assert_eq!(
        std::collections::BTreeSet::from([orig.clone(), orig_01.clone()]),
        actual
    );

    //get all accounts
    let res = client.get(&app_web::PATH_GET_ACCOUNT_LIST).await;
    assert!(res.status() == StatusCode::OK);
    let actual = res.json::<BTreeSet<String>>().await;
    assert_eq!(BTreeSet::from([orig.address, orig_01.address]), actual);
}
