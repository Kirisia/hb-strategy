use std::collections::HashMap;
use hb_buy_strategy::api::HbApi;

#[tokio::main]
async fn main() {
    let api = HbApi::new(
        "xxx",
        "xxx",
        "api.huobi.pro"
    );
    // let resp = api.get_symbols().await.unwrap();
    let mut map: HashMap<&str, String> = HashMap::new();
    map.insert("account-id", "17835863".into());
    map.insert("symbol", "xrpusdt".into());
    map.insert("type", "buy-market".into());
    map.insert("amount", "5".into());
    let resp = api.buy_currency(&map).await.unwrap();
    // let resp = api.get_accounts().await.unwrap();
    let text = resp.text().await.unwrap();
    println!("{}", text);
}