use std::collections::HashMap;
use hb_buy_strategy::api::HbApi;

#[tokio::main]
async fn main() {
    let api = HbApi::new(
        "50d7bf42-bvrge3rf7j-b05b1e00-e53c7",
        "eb5bbf7c-753935db-220bf939-735de",
        "api.huobi.pro"
    );
    let resp = api.get_symbols().await.unwrap();
    let text = resp.text().await.unwrap();
    println!("{}", text);
}