use arc_swap::ArcSwap;
use futures::{channel::oneshot, never::Never};
use log::*;
use rand::{Rng, prelude::StdRng, SeedableRng};
use std::{
    any::Any,
    collections::{HashMap, HashSet},
    net::SocketAddr,
    ops::Range,
    sync::{Arc, Mutex},
    time::Duration, borrow::BorrowMut,
};
use crate::{
    actors::{Actor, ActorPath, Dispatcher, DynActorRef, SystemPath, Transport},
    component::{Component, ComponentContext, ExecuteResult}, prelude::{ComponentLifecycle, Handled, NetMessage, KompactSystem},
    timer::timer_manager::Timer, lookup::{ActorLookup, ActorStore}, messaging::DispatchData,
};
use crate::DynamicPortAccess;
use crate::{timer::{TimerRef, timer_manager::TimerManager}, prelude::{ComponentDefinition}};

/// A simulated network.
pub struct SimulationNetwork {
    //ctx: ComponentContext<SimulationNetwork>,
    rand: StdRng,
    config: Config,
    stat: Stat,
    endpoints: HashMap<SocketAddr, Arc<ArcSwap<ActorStore>>>,
    clogged: HashSet<SocketAddr>,
    clogged_link: HashSet<(SocketAddr, SocketAddr)>,
}
/* 
impl ComponentLifecycle for SimulationNetwork {
    fn on_start(&mut self) -> Handled {
        println!("On start Network");
        Handled::Ok
    }

    fn on_stop(&mut self) -> Handled
    where
        Self: 'static,
    {
        Handled::Ok
    }

    fn on_kill(&mut self) -> Handled
    where
        Self: 'static,
    {
        Handled::Ok
    }
}

impl Actor for SimulationNetwork {
    type Message = Never;

    fn receive_local(&mut self, msg: Self::Message) -> Handled {
        unimplemented!("We are ignoring local messages");
    }

    fn receive_network(&mut self, msg: NetMessage) -> Handled {
        unimplemented!("We are ignoring network messages");
    }
}*/

/// Network configurations.
#[derive(Debug)]
#[allow(missing_docs)]
pub struct Config {
    pub packet_loss_rate: f64,
    pub send_latency: Range<Duration>,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            packet_loss_rate: 0.0,
            send_latency: Duration::from_millis(1)..Duration::from_millis(10),
        }
    }
}

/// Network statistics.
#[derive(Debug, Default, Clone)]
pub struct Stat {
    /// Total number of messages.
    pub msg_count: u64,
    pub system_count: u16
}

impl SimulationNetwork {
    pub fn new() -> Self {
        Self {
            //ctx: ComponentContext::uninitialised(),
            rand: StdRng::seed_from_u64(0),
            config: Config::default(),
            stat: Stat::default(),
            endpoints: HashMap::new(),
            clogged: HashSet::new(),
            clogged_link: HashSet::new(),
        }
    }

    pub fn update_config(&mut self, f: impl FnOnce(&mut Config)) {
        f(&mut self.config);
    }

    pub fn stat(&self) -> &Stat {
        &self.stat
    }

    pub fn register_system(&mut self, mut addr: SocketAddr, lookup: Arc<ArcSwap<ActorStore>>, transport: Transport) -> SystemPath {
        //debug!("insert: {}", target);
        addr.set_port(self.stat.system_count);
        self.stat.system_count += 1; 

        let system_path = SystemPath::new(transport, addr.ip(), addr.port());
        self.endpoints.insert(system_path.socket_address(), lookup);
        system_path
    }

    #[allow(dead_code)]
    pub fn remove(&mut self, target: &SocketAddr) {
        debug!("remove: {}", target);
        self.endpoints.remove(target);
        self.clogged.remove(target);
    }

    pub fn clog(&mut self, target: SocketAddr) {
        assert!(self.endpoints.contains_key(&target));
        debug!("clog: {}", target);
        self.clogged.insert(target);
    }

    pub fn unclog(&mut self, target: SocketAddr) {
        assert!(self.endpoints.contains_key(&target));
        debug!("unclog: {}", target);
        self.clogged.remove(&target);
    }

    pub fn clog_link(&mut self, src: SocketAddr, dst: SocketAddr) {
        assert!(self.endpoints.contains_key(&src));
        assert!(self.endpoints.contains_key(&dst));
        debug!("clog: {} -> {}", src, dst);
        self.clogged_link.insert((src, dst));
    }

    pub fn unclog_link(&mut self, src: SocketAddr, dst: SocketAddr) {
        assert!(self.endpoints.contains_key(&src));
        assert!(self.endpoints.contains_key(&dst));
        debug!("unclog: {} -> {}", src, dst);
        self.clogged_link.remove(&(src, dst));
    }

    pub fn send(&mut self, src: SocketAddr, dst: SocketAddr, data: DispatchData) { // tag: u64, data: Payload
        //trace!("send: {} -> {}, tag={}", src, dst, tag);
        assert!(self.endpoints.contains_key(&src));
        if !self.endpoints.contains_key(&dst)
            || self.clogged.contains(&src)
            || self.clogged.contains(&dst)
            || self.clogged_link.contains(&(src, dst))
        {
            trace!("no connection");
            return;
        }
        if self.rand.gen_bool(self.config.packet_loss_rate) {
            trace!("packet loss");
            return;
        }
        let lookup = self.endpoints[&dst].clone();

        /*let msg = Message {
            tag,
            data,
            from: src,
        };*/

        let latency = self.rand.gen_range(self.config.send_latency.clone());
        trace!("delay: {:?}", latency);

        let msg = data.into_local();

        match msg {
            Ok(envelope) => {
                let lease_lookup = lookup.load();
                match lease_lookup.get_by_actor_path(&envelope.receiver) {
                    crate::lookup::LookupResult::Ref(actor) => actor.enqueue(envelope),
                    crate::lookup::LookupResult::Group(_) => todo!(),
                    crate::lookup::LookupResult::None => todo!(),
                    crate::lookup::LookupResult::Err(_) => todo!(),
                }
                /*Æself.schedule_once(
                    latency,
                    move |_,_| {
                        let lease_lookup = lookup.load();
                        match lease_lookup.get_by_actor_path(&envelope.receiver) {
                            crate::lookup::LookupResult::Ref(actor) => actor.enqueue(envelope),
                            crate::lookup::LookupResult::Group(_) => todo!(),
                            crate::lookup::LookupResult::None => todo!(),
                            crate::lookup::LookupResult::Err(_) => todo!(),
                        }
                        Handled::Ok
                    }
                );*/
            },
            Err(e) => todo!(),
        }
        
        /* .add_timer(self.time.now() + latency, move || {
            ep.lock().unwrap().send(msg);
        });*/
        self.stat.msg_count += 1;
    }

    pub fn recv(&mut self, dst: SocketAddr, tag: u64) -> oneshot::Receiver<Message> {
        //self.endpoints[&dst].lock().unwrap().recv(tag)
        todo!()
    }
}

pub struct Message {
    pub tag: u64,
    pub data: Payload,
    pub from: SocketAddr,
}

pub type Payload = Box<dyn Any + Send + Sync>;

#[derive(Default)]
struct Endpoint {
    registered: Vec<(u64, oneshot::Sender<Message>)>,
    msgs: Vec<Message>,
}

impl Endpoint {
    fn send(&mut self, msg: Message) {
        let mut i = 0;
        let mut msg = Some(msg);
        while i < self.registered.len() {
            if matches!(&msg, Some(msg) if msg.tag == self.registered[i].0) {
                // tag match, take and try send
                let (_, sender) = self.registered.swap_remove(i);
                msg = match sender.send(msg.take().unwrap()) {
                    Ok(_) => return,
                    Err(m) => Some(m),
                };
                // failed to send, try next
            } else {
                // tag mismatch, move to next
                i += 1;
            }
        }
        // failed to match awaiting recv, save
        self.msgs.push(msg.unwrap());
    }

    fn recv(&mut self, tag: u64) -> oneshot::Receiver<Message> {
        let (tx, rx) = oneshot::channel();
        if let Some(idx) = self.msgs.iter().position(|msg| tag == msg.tag) {
            let msg = self.msgs.swap_remove(idx);
            tx.send(msg).ok().unwrap();
        } else {
            self.registered.push((tag, tx));
        }
        rx
    }
}