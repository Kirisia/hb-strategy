use hb_buy_strategy::strategy::{Strategy, StrategyConfig};
use tokio::io::Error;
use hb_buy_strategy::setup_logger;
use std::fs::File;
use std::io::Read;
use log::{info};

#[tokio::main]
async fn main() -> Result<(), Error> {
    std::env::set_var("RUST_LOG", "info");
    setup_logger().unwrap();
    info!("Run!");
    let mut file = File::open("strategy.toml").expect("打开配置文件失败");
    let mut toml_str = String::new();
    file.read_to_string(&mut toml_str).unwrap();
    let config: StrategyConfig = toml::from_str(toml_str.as_str()).expect("加载配置文件失败");
    let mut strategy = Strategy::new(config);
    // println!("{}", strategy.get_current_price().await);
    tokio::spawn(async move {
        loop {
            strategy.run().await;
        }
    }).await?;
    Ok(())
}
