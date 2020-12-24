use std::fs::File;
use std::io::Read;
use hb_buy_strategy::strategy::StrategyConfig;

fn main() {
    let mut file = File::open("strategy.toml").expect("打开配置文件失败");
    let mut toml_str = String::new();
    file.read_to_string(&mut toml_str).unwrap();
    println!("{}", toml_str);
    let config: StrategyConfig = toml::from_str(toml_str.as_str()).expect("加载配置文件失败");
    println!("{:?}", config);
    println!("{}", 2.0f64.powi(1))
}