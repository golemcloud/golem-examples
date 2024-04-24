use reqwest::*;
use serde::{Deserialize, Serialize};
use std::env;

use crate::model::*;

pub fn create(auction: Auction) {
    create_worker(auction.auction_id.clone());
    let invocation_key = get_invocation_key(auction.auction_id.clone());
    initialize_worker(auction, invocation_key);
}

fn create_worker(auction_id: AuctionId) {
    let client = Client::new();
    let component = env::var("AUCTION_COMPONENT_ID").unwrap();
    let url = format!(
        "https://release.api.golem.cloud/v1/templates/{}/workers",
        component
    );
    let body = CreateWorkerBody::new(format!("auction-{}", auction_id.auction_id));
    let token = env::var("GOLEM_TOKEN_SECRET").unwrap();
    let response = client
        .post(url)
        .json(&body)
        .header("Authorization", format!("Bearer {}", token))
        .send()
        .unwrap();
    assert!(response.status().is_success());
}

fn get_invocation_key(auction_id: AuctionId) -> InvocationKey {
    let client = Client::new();
    let component = env::var("AUCTION_COMPONENT_ID").unwrap();
    let worker_id = format!("auction-{}", auction_id.auction_id);
    let url = format!(
        "https://release.api.golem.cloud/v1/templates/{}/workers/{}/key",
        component, worker_id
    );
    let token = env::var("GOLEM_TOKEN_SECRET").unwrap();
    let response = client
        .post(url)
        .header("Authorization", format!("Bearer {}", token))
        .send()
        .unwrap();
    assert!(response.status().is_success());
    response.json().unwrap()
}

fn initialize_worker(auction: Auction, invocation_key: InvocationKey) {
    let client = Client::new();
    let component = env::var("AUCTION_COMPONENT_ID").unwrap();
    let worker_id = format!("auction-{}", auction.auction_id.auction_id);
    let url = format!("https://release.api.golem.cloud/v1/templates/{}/workers/{}/invoke-and-await", component, worker_id);
    let body = InitializeWorkerBody::new(auction);
    let token = env::var("GOLEM_TOKEN_SECRET").unwrap();
    let function = "golem:template/api/initialize";
    let query_params = InitializeWorkerQueryParams::new(invocation_key, function.to_string());
    let response = client
        .post(url)
        .json(&body)
        .header("Authorization", format!("Bearer {}", token))
        .query(&query_params)
        .send()
        .unwrap();
    assert!(response.status().is_success());
}

#[derive(Serialize, Deserialize, Debug)]
struct CreateWorkerBody {
    name: String,
    args: Vec<String>,
    env: Vec<Vec<String>>,
}

impl CreateWorkerBody {
    fn new(name: String) -> CreateWorkerBody {
        CreateWorkerBody {
            name,
            args: Vec::new(),
            env: Vec::new(),
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
struct InvocationKey {
    value: String,
}

#[derive(Serialize, Deserialize, Debug)]
struct InitializeWorkerBody {
    params: Vec<InitializeWorkerParams>,
}

impl InitializeWorkerBody {
    fn new(auction: Auction) -> InitializeWorkerBody {
        InitializeWorkerBody {
            params: vec!(InitializeWorkerParams {
                auction_id: AuctionIdParam::new(auction.auction_id.clone()),
                name: auction.name,
                description: auction.description,
                limit_price: auction.limit_price,
                expiration: auction.expiration.deadline.as_secs(),
            }),
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
struct InitializeWorkerParams {
    #[serde(rename = "auction-id")]
    auction_id: AuctionIdParam,
    name: String,
    description: String,
    #[serde(rename = "limit-price")]
    limit_price: f32,
    expiration: u64,
}

#[derive(Serialize, Deserialize, Debug)]
struct AuctionIdParam {
    #[serde(rename = "auction-id")]
    auction_id: String,
}

impl AuctionIdParam {
    fn new(auction_id: AuctionId) -> AuctionIdParam {
        AuctionIdParam {
            auction_id: auction_id.auction_id.to_string(),
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
struct InitializeWorkerQueryParams {
    #[serde(rename = "invocation-key")]
    invocation_key: String,
    function: String,
}

impl InitializeWorkerQueryParams {
    fn new(invocation_key: InvocationKey, function: String) -> InitializeWorkerQueryParams {
        InitializeWorkerQueryParams {
            invocation_key: invocation_key.value,
            function,
        }
    }
}
