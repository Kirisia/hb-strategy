use std::collections::HashMap;
use actix::{Recipient, Actor};
use actix::prelude::*;
use tokio::runtime::Handle;
use rand::rngs::ThreadRng;
use rand::Rng;
use crate::strategy::{Strategy, StrategyConfig};
use crate::proto::SendParcel;
use std::sync::{Arc, RwLock};

lazy_static! {
    static ref STRATEGY: Arc<RwLock<Strategy>> = Arc::new(RwLock::new(Strategy::new(StrategyConfig::default())));
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct Message(pub String);

pub struct StrategyServer {
    sessions: HashMap<usize, Recipient<Message>>, // 房间长连接
    rng: ThreadRng,
    running: bool,
}

impl Default for StrategyServer {
    fn default() -> Self {
        Self {
            sessions: HashMap::new(),
            rng: rand::thread_rng(),
            running: false,
        }
    }
}

impl StrategyServer {
    fn send_message(&self, message: SendParcel) {
        let message = serde_json::to_string(&message).unwrap();
        while let Some((_, addr)) = self.sessions.iter().next() {
            let _ = addr.do_send(Message(message.to_owned()));
        }
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
        if self.running {
            let strategy = (*STRATEGY).read().unwrap();
            self.send_message(SendParcel::StrategyState(strategy.get_state_string()));
            return
        }
        let addr = ctx.address();
        let rule = msg.0;
        match toml::from_str(&rule) {
            Ok(config) => {
                let mut strategy = (*STRATEGY).write().unwrap();
                strategy.set_config(config);
                addr.do_send(Frame);
                actix::spawn(async move {
                    strategy.run().await;
                });
            }
            Err(_) => {
                self.send_message(SendParcel::ConfigError("加载配置错误！".into()))
            }
        }

    }
}

#[derive(Message)]
#[rtype(isize)]
struct Frame;

impl Handler<Frame> for StrategyServer {
    type Result = isize;

    fn handle(&mut self, msg: Frame, _: &mut Context<Self>) -> Self::Result {
        // self.strategy.run();
        1
    }
}
