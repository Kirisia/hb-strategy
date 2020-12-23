use serde_json::Value;
use reqwest;
use tokio::time::Duration;
use log::{info, error};
use reqwest::Response;
use std::fs::OpenOptions;
use std::io::Write;

pub struct StrategyConfig {
    pub first_amount: f64, // 首单买入价格
    pub double_cast: u8, // 倍投上限
    // 止盈比例
    pub spr: f64,
    pub profit_cb: f64, // 盈利回调
    pub cover_decline: f64, // 补仓跌幅
    pub cover_cb: f64, // 补仓回调
    pub currency: String, // 币种
}

pub struct Strategy {
    config: StrategyConfig,
    hold_amount: f64, // 持仓金额
    average_amount: f64, // 持仓均价
    cover_num: u8, // 补仓次数
    hold_num: f64, // 持仓数量
    current_amount: f64, // 当前价格
    increase_amp: f64, // 涨幅
}

impl Strategy {
    pub fn new(config: StrategyConfig) -> Self {
        Self {
            config,
            hold_amount: 0.0,
            average_amount: 0.0,
            cover_num: 0,
            hold_num: 0.0,
            current_amount: 0.0,
            increase_amp: 0.0,
        }
    }
    pub async fn get_current_price(&self) -> f64 {
        let url = format!("https://api.huobi.pro/market/history/kline?period=1min&symbol={}usdt&size=1", self.config.currency);
        let mut resp: Option<Response> = None;
        loop {
            resp = match reqwest::get(&url).await {
                Ok(resp) => Some(resp),
                Err(err) => {
                    error!("{:?}", err);
                    None
                }
            };
            if resp.is_some() {
                let rep = resp.unwrap();
                let json = rep.json::<Value>().await.unwrap();
                let vc = json.get("data").unwrap().as_array().unwrap();
                if !vc.is_empty() {
                    let v = &vc[0];
                    return v.get("close").unwrap().as_f64().unwrap()
                }
                resp = None;
            }
        }
    }

    pub async fn run(&mut self) {
        let config = &self.config;
        self.hold_amount = config.first_amount;
        self.current_amount = self.get_current_price().await;
        self.hold_num = self.hold_amount / self.current_amount;

        let (mut profit, mut high_profit, mut cover, mut low_cover )
            = (false, 0.0, false, f64::MAX);
        loop {
            self.current_amount = self.get_current_price().await;
            self.average_amount = self.hold_amount / self.hold_num;
            info!(
                "\n持仓金额 = {}, 持仓均价 = {}, 补仓次数 = {} \n持仓数量 = {}, 当前价格 = {}",
                self.hold_amount, self.average_amount, self.cover_num, self.hold_num, self.current_amount
            );
            // 判断是否盈利
            if (self.current_amount - self.average_amount) / self.average_amount > config.spr {
                if self.current_amount > high_profit {
                    high_profit = self.current_amount;
                }
                profit = true;
            }
            if profit {
                if (high_profit - self.current_amount) / self.current_amount > config.profit_cb {
                    // 盈利策略结束
                    let profit_amount = self.current_amount * self.hold_num - self.hold_amount;
                    profit_log(format!("盈利 = {}", profit_amount));
                    info!("盈利策略结束！盈利 = {}", profit_amount);
                    break;
                }
            }
            // 判断是否需要补仓
            if (self.average_amount - self.current_amount) / self.average_amount > config.cover_decline {
                if low_cover > self.current_amount {
                    low_cover = self.current_amount;
                }
                cover = true;
            }
            if cover {
                if (self.current_amount - low_cover) / low_cover > config.cover_cb {
                    // 开始补仓
                    if self.cover_num < config.double_cast {
                        info!("开始补仓！");
                        self.cover_num += 1;
                        let buy_in = 2.0 * self.hold_amount;
                        self.hold_amount += buy_in;
                        self.hold_num += buy_in / self.current_amount;
                    }
                }
            }
            tokio::time::delay_for(Duration::new(5, 0)).await;
        }
        // println!("{}", self.hold_num);
    }

    pub async fn loop_run(&mut self) {
        loop {
            self.run();
        }
    }
}

fn profit_log<S: Into<String>>(str: S) {
    let mut file = OpenOptions::new()
        .append(true)
        .create(true)
        .open("profit.log")
        .expect("打开文件失败");
    let str = format!("{} {}\n", chrono::Local::now().format("[%Y-%m-%d][%H:%M:%S]"), str.into());
    file.write(str.as_bytes()).unwrap();
}
