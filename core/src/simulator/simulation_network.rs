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

use kompact_actor_derive::Actor;
use crate::ActorRaw;

pub struct SimulationNetwork {
    rand: StdRng,
    config: Config,
    stat: Stat,
    endpoints: HashMap<SocketAddr, Arc<ArcSwap<ActorStore>>>,
    clogged: HashSet<SocketAddr>,
    clogged_link: HashSet<(SocketAddr, SocketAddr)>,
}

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

#[derive(Debug, Default, Clone)]
pub struct Stat {
    pub msg_count: u64,
    pub system_count: u16
}

impl SimulationNetwork {
    pub fn new() -> Self {
        Self {
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
        println!("addr1: {} addr2: {}", src.to_string(), dst.to_string());
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

    pub fn send(&mut self, src: SocketAddr, dst: SocketAddr, data: DispatchData) {
        //println!("senddd addr1: {} addr2: {} clogged: {}", src.to_string(), dst.to_string(), self.clogged_link.contains(&(src, dst)));
        //println!("contains clogged link?: {} ", self.clogged_link.contains(&(src, dst)));
        assert!(self.endpoints.contains_key(&src));
        if !self.endpoints.contains_key(&dst)
            || self.clogged.contains(&src)
            || self.clogged.contains(&dst)
            || self.clogged_link.contains(&(src, dst))
        {
            println!("Message from: {} to: {} not sent due to broken link", src.to_string(), dst.to_string());
            trace!("no connection");
            return;
        }
        if self.rand.gen_bool(self.config.packet_loss_rate) {
            trace!("packet loss");
            return;
        }
        let lookup = self.endpoints[&dst].clone();

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
                /*self.schedule_once(
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
        self.stat.msg_count += 1;
    }
}
