use serde_json::Value;
use reqwest;
use tokio::time::Duration;
use log::{info, error};
use reqwest::Response;
use std::fs::OpenOptions;
use std::io::Write;
use serde::{Deserialize};
use crate::api::HbApi;
use std::collections::HashMap;

#[derive(Deserialize, Debug)]
pub struct StrategyConfig {
    pub first_amount: f64, // 首单买入价格
    pub double_cast: u8, // 倍投上限
    // 止盈比例
    pub spr: f64,
    pub profit_cb: f64, // 盈利回调
    pub cover_decline: f64, // 补仓跌幅
    pub cover_cb: f64, // 补仓回调
    pub currency: String, // 币种
    pub sleep: u64, // 策略频率
    pub access_key: String,
    pub secret_key: String,
    pub mock: bool, // 是否使用模拟数据
}

impl Default for StrategyConfig {
    fn default() -> Self {
        Self {
            first_amount: 0.0,
            double_cast: 0,
            spr: 0.0,
            profit_cb: 0.0,
            cover_decline: 0.0,
            cover_cb: 0.0,
            currency: "".into(),
            sleep: 0,
            mock: true,
            access_key: "".into(),
            secret_key: "".into(),
        }
    }
}

pub struct Strategy {
    config: StrategyConfig,
    hold_amount: f64, // 持仓金额
    average_amount: f64, // 持仓均价
    cover_num: u8, // 补仓次数
    hold_num: f64, // 持仓数量
    current_amount: f64, // 当前价格
    increase_amp: f64, // 涨幅
    profit_per: f64, // 盈利比例
    is_profit: bool, // 是否盈利
    high_profit: f64, // 最高盈利
    is_cover: bool, // 是否需要补仓
    low_cover: f64, // 最低补仓
    api: HbApi,
    pub is_running: bool,
    account_id: String,
}

impl Strategy {
    pub fn new(config: StrategyConfig) -> Self {
        Self {
            config,
            hold_amount: 0.0,
            average_amount: 0.0,
            hold_num: 0.0,
            current_amount: 0.0,
            increase_amp: 0.0,
            cover_num: 0,
            profit_per: 0.0,
            is_profit: false,
            high_profit: 0.0,
            is_cover: false,
            low_cover: f64::MAX,
            is_running: false,
            account_id: "".into(),
            api: HbApi::new("", "", "api.huobi.pro"),
        }
    }

    pub async fn get_account_id(&mut self) {
        match self.api.get_accounts().await {
            Ok(resp) => {
                let v = resp.json::<Value>().await.unwrap();
                let arr = v["data"].as_array().unwrap();
                for v in arr {
                    if v["type"].as_str().unwrap() == "spot" {
                        self.account_id = v["id"].as_str().unwrap().to_string();
                        break
                    }
                }
            },
            Err(_) => {
                error!("获取子账号id错误。");
                panic!("获取子账号id错误")
            }
        };
    }

    pub async fn buy_currency(&mut self, amount: f64) {
        let mut map: HashMap<&str, String> = HashMap::new();
        map.insert("account-id", self.account_id.clone());
        map.insert("symbol", format!("{}usdt", self.config.currency));
        map.insert("type", "buy-market".into());
        map.insert("amount", amount.to_string());
        let order_id = match self.api.order_place(&map).await {
            Ok(resp) => {
                let v = resp.json::<Value>().await.unwrap();
                v["data"].as_str().unwrap().to_string()
            },
            Err(_) => {
                error!("买入失败！");
                panic!("买币失败");
            }
        };
        let resp = self.api.http_get(
            format!("/v1/order/orders/{}", order_id), &HashMap::new()
        )
            .await
            .unwrap();
        let v = resp.json::<Value>().await.unwrap();
        self.hold_amount += v["data"]["field-cash-amount"].as_f64().unwrap();
        self.hold_num += v["data"]["field-amount"].as_f64().unwrap();
    }

    pub async fn sell_currency(&self) -> f64 {
        let mut map: HashMap<&str, String> = HashMap::new();
        map.insert("account-id", self.account_id.clone());
        map.insert("symbol", format!("{}usdt", self.config.currency));
        map.insert("type", "sell-market".into());
        map.insert("amount", self.hold_num.to_string());
        let order_id = match self.api.order_place(&map).await {
            Ok(resp) => {
                info!("卖出成功！");
                let v = resp.json::<Value>().await.unwrap();
                v["data"].as_str().unwrap().to_string()
            },
            Err(_) => {
                error!("卖出失败！");
                panic!("卖币失败！");
            }
        };
        let resp = self.api.http_get(
            format!("/v1/order/orders/{}", order_id), &HashMap::new()
        )
            .await
            .unwrap();
        let v = resp.json::<Value>().await.unwrap();
        let amount = v["data"]["field-cash-amount"].as_f64().unwrap();
        let profit_amount = amount - self.hold_num;
        profit_log(format!("策略结束！盈利 = {}", profit_amount));
        info!("策略结束！盈利 = {}", profit_amount);
        profit_amount
    }

    pub async fn reset(&mut self) {
        let config = &self.config;
        self.api = HbApi::new(config.access_key.as_str(), config.secret_key.as_str(), "api.huobi.pro");
        self.current_amount = self.get_current_price().await;
        if config.mock {
            self.hold_amount = config.first_amount;
            self.hold_num = self.hold_amount / self.current_amount;
        } else {
            let first_amount = config.first_amount;
            self.get_account_id().await;
            self.buy_currency(first_amount).await;
        }
        self.cover_num = 0;
        self.is_profit = false;
        self.high_profit = 0.0;
        self.is_cover = false;
        self.low_cover = f64::MAX;
    }

    pub fn set_config(&mut self, config: StrategyConfig) {
        self.config = config;
    }

    pub async fn get_current_price(&self) -> f64 {
        // let url = format!("https://api.huobi.pro/market/history/kline?period=1min&symbol={}usdt&size=1", self.config.currency);
        let mut resp: Option<Response>;
        loop {
            resp = match self.api.get_current_amount(&self.config.currency).await {
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
            }
        }
    }

    pub async fn run(&mut self) {
        self.reset().await;
        loop {
            self.step().await;
            tokio::time::delay_for(Duration::new(self.config.sleep.max(1), 0)).await;
        }
        // println!("{}", self.hold_num);
    }

    pub async fn step(&mut self) -> f64 {
        let config = &self.config;
        self.current_amount = self.get_current_price().await;
        self.average_amount = self.hold_amount / self.hold_num;
        // 判断是否盈利
        // 盈利比例
        let profit_per = (self.current_amount - self.average_amount) / self.average_amount;
        self.profit_per = profit_per;
        info!(
            "\n持仓金额 = {}, 持仓均价 = {}, 补仓次数 = {} \n持仓数量 = {}, 当前价格 = {}, 盈利比例 = {}",
            self.hold_amount, self.average_amount, self.cover_num,
            self.hold_num, self.current_amount, profit_per
        );

        if profit_per > config.spr {
            if profit_per > self.high_profit {
                self.high_profit = profit_per;
            }
            self.is_profit = true;
            self.is_cover = false;
        } else {
            self.is_profit = false;
        }
        if self.is_profit {
            if self.high_profit - profit_per > config.profit_cb {
                // 盈利策略结束
                return if config.mock {
                    let profit_amount = self.current_amount * self.hold_num - self.hold_amount;
                    profit_log(format!("策略结束！盈利 = {}", profit_amount));
                    info!("策略结束！盈利 = {}", profit_amount);
                    self.is_running = false;
                    profit_amount
                } else {
                    self.is_running = false;
                    self.sell_currency().await
                }
            }
        }
        // 判断是否需要补仓
        if (self.average_amount - self.current_amount) / self.average_amount > config.cover_decline {
            if self.low_cover > self.current_amount {
                self.low_cover = self.current_amount;
            }
            self.is_cover = true;
        }
        if self.is_cover {
            if (self.current_amount - self.low_cover) / self.low_cover > config.cover_cb {
                // 开始补仓
                if self.cover_num < config.double_cast {
                    info!("开始补仓！");
                    self.cover_num += 1;
                    let buy_in = 2.0f64.powi(self.cover_num as i32) * config.first_amount;
                    if config.mock {
                        self.hold_amount += buy_in;
                        self.hold_num += buy_in / self.current_amount;
                    } else {
                        self.buy_currency(buy_in).await;
                    }
                    self.is_cover = false;
                }
            }
        }
        -1.0
    }

    pub fn get_state_string(&self) -> String {
        format!(
            "{}\n持仓金额 = {}, 持仓均价 = {}, 补仓次数 = {} \n持仓数量 = {}, 当前价格 = {}, 盈利比例 = {}",
            chrono::Local::now().format("[%Y-%m-%d][%H:%M:%S]"),
            self.hold_amount, self.average_amount, self.cover_num,
            self.hold_num, self.current_amount, self.profit_per
        )
    }

    // pub async fn loop_run(&mut self) {
    //     loop {
    //         self.run();
    //     }
    // }
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
