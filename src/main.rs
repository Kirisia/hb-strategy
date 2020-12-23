use serde_json::{Value};
use std::time::Instant;
use hb_buy_strategy::strategy::{Strategy, StrategyConfig};
use tokio::io::Error;
use hb_buy_strategy::setup_logger;

#[tokio::main]
async fn main() -> Result<(), Error> {
    std::env::set_var("RUST_LOG", "info");
    setup_logger().unwrap();
    let (limit, dcl, tpr, profit_cb, decline, decline_cb) =
        (300.0, 7, 0.003, 0.003, 0.01, 0.001);
    let mut strategy = Strategy::new(StrategyConfig {
        first_amount: limit,
        double_cast: dcl,
        spr: tpr,
        profit_cb: profit_cb,
        cover_decline: decline,
        cover_cb: decline_cb,
        currency: "eth".into(),
    });
    // println!("{}", strategy.get_current_price().await);
    tokio::spawn(async move {
        loop {
            strategy.run().await;
        }
    }).await?;
    Ok(())
}
