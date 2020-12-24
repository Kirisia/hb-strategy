use std::collections::HashMap;
use hb_buy_strategy::api::HbApi;

#[tokio::main]
async fn main() {
    let api = HbApi::new(
        "xxx",
        "xxx",
        "api.huobi.pro"
    );
    let resp = api.get_symbols().await.unwrap();
    let text = resp.text().await.unwrap();
    println!("{}", text);
}