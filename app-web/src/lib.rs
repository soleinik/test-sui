use app_data::BalanceChange;
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use tokio::sync::RwLock;
//use tokio::sync::RwLock;

use std::{collections::BTreeMap, sync::Arc};

#[derive(Default, Clone)]
pub struct AppState {
    pub db: Arc<RwLock<BTreeMap<String, BalanceChange>>>,
}

/*

curl -s  http://localhost:8080/balances | jq
curl -s  http://localhost:8080/balances/0x7d20dcdb2bca4f508ea9613994683eb4e76e9c4ed371169677c1be02aaf0b58e | jq

curl -s  http://localhost:8080/accounts | jq


*/

pub const PATH_POST_BALANCE: &str = "/balances";
pub const PATH_GET_BALANCE_LIST: &str = "/balances";
pub const PATH_GET_BALANCE_ACCOUNT: &str = "/balances/:account_id";
pub const PATH_GET_ACCOUNT_LIST: &str = "/accounts";

pub fn router() -> Router {
    Router::new()
        .route(PATH_POST_BALANCE, post(balance_change_set))
        .route(PATH_GET_BALANCE_LIST, get(balance_list_get))
        .route(PATH_GET_BALANCE_ACCOUNT, get(account))
        .route(PATH_GET_ACCOUNT_LIST, get(account_list_get))
        .with_state(Arc::new(AppState::default()))
}

pub async fn run_web() {
    // run our app with hyper, listening globally on port 3000
    let listener = tokio::net::TcpListener::bind("0.0.0.0:8080").await.unwrap();
    axum::serve(listener, router()).await.unwrap();
}

async fn balance_change_set(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<Vec<BalanceChange>>,
) -> impl IntoResponse {
    println!("{payload:#?}");
    let mut lock = state.db.write().await;
    payload.into_iter().for_each(|b| {
        //not sure what the proper logic should be here
        if let Some(old) = lock.get_mut(&b.address) {
            old.amount += b.amount;
        } else {
            lock.insert(b.address.clone(), b);
        }
    });

    StatusCode::CREATED
}

async fn balance_list_get(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    let lock = state.db.read().await;
    (
        StatusCode::OK,
        Json(
            lock.iter()
                .map(|(_, v)| v)
                .cloned()
                .collect::<Vec<BalanceChange>>(),
        ),
    )
}

async fn account(
    Path(account_id): Path<String>,
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    let lock = state.db.read().await;
    if let Some(r) = lock.get(&account_id) {
        (StatusCode::OK, Json(Some(r.to_owned())))
    } else {
        (StatusCode::NOT_FOUND, Json(None))
    }
}

async fn account_list_get(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    let lock = state.db.read().await;
    (
        StatusCode::OK,
        Json(lock.keys().cloned().collect::<Vec<String>>()),
    )
}
