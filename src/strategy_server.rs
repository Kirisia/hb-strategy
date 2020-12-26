use std::collections::HashMap;
use actix::{Recipient, Actor};
use actix::prelude::*;
use tokio::runtime::Handle;
use rand::rngs::ThreadRng;
use rand::Rng;
use crate::strategy::{Strategy, StrategyConfig};
use crate::proto::SendParcel;
use std::sync::{Arc, RwLock};
use actix::clock::Duration;
use std::ops::Deref;
use log::{info};
use std::fs::File;
use std::io::{Seek, SeekFrom, Read};

lazy_static! {
    static ref STRATEGY: Arc<RwLock<Strategy>> = Arc::new(RwLock::new(Strategy::new(StrategyConfig::default())));
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct Message(pub String);

pub struct StrategyServer {
    sessions: HashMap<usize, Recipient<Message>>, // 长连接消息通道
    rng: ThreadRng,
}

impl Default for StrategyServer {
    fn default() -> Self {
        Self {
            sessions: HashMap::new(),
            rng: rand::thread_rng(),
        }
    }
}

impl StrategyServer {
    fn send_message(&self, message: SendParcel) {
        let message = serde_json::to_string(&message).unwrap();
        for (_, addr) in &self.sessions {
            addr.do_send(Message(message.to_owned()));
        }
        // while let Some((_, addr)) = self.sessions.iter().next() {
        //     let _ = addr.do_send(Message(message.to_owned()));
        // }
    }
}

impl Actor for StrategyServer {
    type Context = Context<Self>;
}

#[derive(Message)]
#[rtype(usize)]
pub struct Connect {
    pub addr: Recipient<Message>,
}

impl Handler<Connect> for StrategyServer {
    type Result = usize;

    fn handle(&mut self, msg: Connect, ctx: &mut Context<Self>) -> Self::Result {

        let id = self.rng.gen::<u32>() as usize;
        self.sessions.insert(id, msg.addr);
        id
    }
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct Disconnect {
    pub id: usize,
}

impl Handler<Disconnect> for StrategyServer {
    type Result = ();

    fn handle(&mut self, msg: Disconnect, _: &mut Context<Self>) -> Self::Result {
        self.sessions.remove(&msg.id);
    }
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct StrategyMessage(pub String);

impl Handler<StrategyMessage> for StrategyServer {
    type Result = ();

    fn handle(&mut self, msg: StrategyMessage, ctx: &mut Context<Self>) -> Self::Result {
        let strategy = (*STRATEGY).read().unwrap();
        if strategy.is_running {
            self.send_message(SendParcel::StrategyState(read_log()));
            return
        }
        let addr = ctx.address();
        let rule = msg.0;
        actix::spawn(async move {
            match toml::from_str::<StrategyConfig>(&rule) {
                Ok(config) => {
                    let mut strategy = (*STRATEGY).write().unwrap();
                    let sleep = config.sleep;
                    strategy.set_config(config);
                    strategy.is_running = true;
                    strategy.reset().await;
                    info!("running");
                    drop(strategy);
                    loop {
                        let mut strategy = (*STRATEGY).write().unwrap();
                        if !strategy.is_running { break }
                        let profit_amount = strategy.step().await;
                        if profit_amount >= 0.0 {
                            addr.do_send(
                                LogMessage(
                                    SendParcel::StrategyState(
                                        format!("策略结束！盈利 = {}", profit_amount)
                                    )
                                )
                            );
                            break
                        }
                        addr.do_send(
                            LogMessage(
                                SendParcel::StrategyState(read_log())
                            )
                        );
                        drop(strategy);
                        actix::clock::delay_for(Duration::new(sleep.max(1), 0)).await;
                    }
                }
                Err(_) => {
                    addr.do_send(
                        LogMessage(
                            SendParcel::ConfigError("加载配置错误！".into())
                        )
                    )
                    // self.send_message(SendParcel::ConfigError("加载配置错误！".into()))
                }
            }
        });
    }
}

#[derive(Message)]
#[rtype(isize)]
struct Frame;

impl Handler<Frame> for StrategyServer {
    type Result = isize;

    fn handle(&mut self, msg: Frame, _: &mut Context<Self>) -> Self::Result {
        1
    }
}

#[derive(Message)]
#[rtype(result = "()")]
struct LogMessage(pub SendParcel);

impl Handler<LogMessage> for StrategyServer {
    type Result = ();

    fn handle(&mut self, msg: LogMessage, _: &mut Context<Self>) -> Self::Result {
        self.send_message(msg.0);
    }
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct StopStrategy;

impl Handler<StopStrategy> for StrategyServer {
    type Result = ();

    fn handle(&mut self, msg: StopStrategy, _: &mut Context<Self>) -> Self::Result {
        let mut strategy = (*STRATEGY).write().unwrap();
        strategy.is_running = false;
        info!("已停止策略!");
        self.send_message(SendParcel::ConfigError(read_log()));
    }
}

fn read_log() -> String {
    let mut file = File::open("output.log").unwrap();
    let mut str = String::new();
    file.read_to_string(&mut str);
    let mut log = String::new();
    let mut index = 0;
    for line in str.lines().rev() {
        log = format!("{}\n{}", line, log);
        if index > 30 { break }
        index += 1;
    }
    log
}