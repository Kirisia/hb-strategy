use hb_buy_strategy::strategy::{Strategy, StrategyConfig};
use tokio::io::Error;
use hb_buy_strategy::setup_logger;
use std::fs::File;
use std::io::Read;
use log::{info};
use actix_web::{HttpServer, web, App};
use actix_web::middleware::Logger;
use std::thread::sleep;
use actix::clock::Duration;
use hb_buy_strategy::strategy_server::StrategyServer;
use actix::Actor;
use hb_buy_strategy::ws::ws_index;

// #[tokio::main]
// async fn main() -> Result<(), Error> {
//     std::env::set_var("RUST_LOG", "info");
//     setup_logger().unwrap();
//     info!("Run!");
//     let mut file = File::open("strategy.toml").expect("打开配置文件失败");
//     let mut toml_str = String::new();
//     file.read_to_string(&mut toml_str).unwrap();
//     let config: StrategyConfig = toml::from_str(toml_str.as_str()).expect("加载配置文件失败");
//     let mut strategy = Strategy::new(config);
//     // println!("{}", strategy.get_current_price().await);
//     tokio::spawn(async move {
//         loop {
//             strategy.run().await;
//         }
//     }).await?;
//     Ok(())
// }


#[actix_rt::main]
async fn main() -> std::io::Result<()> {
    std::env::set_var("RUST_LOG", "info");
    setup_logger().unwrap();

    // Start chat server actor
    let server = StrategyServer::default().start();

    HttpServer::new(move || {
        App::new()
            // enable logger
            .wrap(Logger::default())
            .data(server.clone())
            // websocket route
            .service(web::resource("/ws").route(web::get().to(ws_index)))
    })
        // start http server on 127.0.0.1:8080
        .bind("0.0.0.0:8778")?
        .run()
        .await
}