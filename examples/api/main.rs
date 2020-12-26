use std::collections::HashMap;
use hb_buy_strategy::api::HbApi;
use serde_json::Value;

#[tokio::main]
async fn main() {
    let api = HbApi::new(
        "5313cf66-xa2b53ggfc-b4a8f9ab-5b81b",
        "8d893168-d59961c0-08adec29-2c2f9",
        "api.huobi.pro"
    );
    // let resp = api.get_symbols().await.unwrap();
    let mut map: HashMap<&str, String> = HashMap::new();
    map.insert("account-id", "17835863".into());
    map.insert("symbol", "xrpusdt".into());
    map.insert("type", "sell-market".into());
    map.insert("amount", "6".into());
    // let resp = api.order_place(&map).await.unwrap();
    // let v = resp.json::<Value>().await.unwrap();
    // let order_id = v["data"].as_str().unwrap();
    let resp = api.http_get(format!("/v1/order/orders/{}", "175326444115697"), &HashMap::new()).await.unwrap();
    // let resp = api.get_accounts().await.unwrap();
    let text = resp.text().await.unwrap();
    println!("{}", text);
}