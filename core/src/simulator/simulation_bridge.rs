use super::*;
use actors::Transport;
use arc_swap::ArcSwap;
use dispatch::lookup::ActorStore;
use futures::task::waker;

use crate::{
    dispatch::NetworkStatus,
    messaging::{DispatchData, DispatchEnvelope, EventEnvelope},
    net::{events::{DispatchEvent, self}, frames::*, network_thread::NetworkThreadBuilder, Protocol, NetworkBridgeErr},
    prelude::{NetworkConfig, SimulationNetworkConfig},
};
use crossbeam_channel::{unbounded as channel, RecvError, SendError, Sender};
use ipnet::IpNet;
use mio::{Interest, Waker};
pub use std::net::SocketAddr;
use std::{io, net::IpAddr, panic, sync::Arc, thread, time::Duration, alloc::System};
use uuid::Uuid;
use simulation_network::SimulationNetwork;

#[derive(Clone)]
pub struct SimulationBridge {
    /// Network-specific configuration
    //cfg: BridgeConfig,
    /// Core logger; shared with network thread
    log: KompactLogger,
    /// Shared actor reference lookup table
    // lookup: Arc<ArcSwap<ActorStore>>,
    /// Network Thread stuff:
    // network_thread: Box<NetworkThread>,
    // ^ Can we avoid storing this by moving it into itself?
    /// Tokio Runtime
    // tokio_runtime: Option<Runtime>,
    /// Reference back to the Kompact dispatcher
    /// Socket the network actually bound on
    bound_address: SocketAddr,
    system_path: SystemPath,
    network: Arc<Mutex<SimulationNetwork>>,
}

impl SimulationBridge {
    pub fn new(
        lookup: Arc<ArcSwap<ActorStore>>,
        network_log: KompactLogger,
        bridge_log: KompactLogger,
        addr: SocketAddr,
        network_config: &SimulationNetworkConfig,
        network: Arc<Mutex<SimulationNetwork>>,
        transport: Transport
    ) -> (Self, SocketAddr) {
        let syspath = SystemPath::new(transport, addr.ip(), addr.port());
        network.lock().unwrap().register_system_path(addr, syspath.clone());

        let bridge = SimulationBridge {
            log: bridge_log,
            bound_address: addr,
            system_path: syspath,
            network,
        };

        (bridge, addr)
    }

    pub fn set_system_path(&mut self, syspath: SystemPath){
        self.system_path = syspath.clone();
        println!("Setting path");
        self.network.lock().unwrap().register_system_path(self.bound_address, syspath);
    }

    /// Returns the local address if already bound
    pub fn local_addr(&self) -> SocketAddr {
        self.bound_address
    }

    /*pub fn get_actor_store(&self) -> SocketAddr {
        self.bound_address
    }*/

    /// Sets the dispatcher reference, returning the previously stored one
    /*pub fn set_dispatcher(&mut self, dispatcher: DispatcherRef) -> Option<DispatcherRef> {
        std::mem::replace(&mut self.dispatcher, Some(dispatcher))
    }*/

    /// Forwards `serialized` to the NetworkThread and makes sure that it will wake up.
    pub(crate) fn route(
        &self,
        addr: SocketAddr,
        data: DispatchData,
        protocol: Protocol,
    ) -> Result<(), NetworkBridgeErr> {
        println!("ROUTE IN BRIDGE");
        match protocol {
            Protocol::Tcp => {
                println!("tcp????????");
                self.network.lock().unwrap().send(self.system_path(), DispatchEvent::SendTcp(addr, data));
            }
            Protocol::Udp => {
                println!("UDP????????");
                self.network.lock().unwrap().send(self.system_path(), DispatchEvent::SendUdp(addr, data));
            }
        }
        Ok(())
    }

    fn system_path(&self) -> SystemPath {
        self.system_path.clone()
        /* 
        match self.system_path {
            Some(ref path) => path.clone(),
            None => {
                let bound_addr = self.local_addr();
                let sp = SystemPath::new(self.transport, bound_addr.ip(), bound_addr.port());
                self.system_path = Some(sp.clone());
                sp
            }
        }*/
    }

    pub fn connect(&self, proto: Transport, addr: SocketAddr) -> Result<(), NetworkBridgeErr> {
        todo!();
        /*match proto {
            Transport::Tcp => {
                self.network_input_queue
                    .send(events::DispatchEvent::Connect(addr))?;
                self.waker.wake()?;
                Ok(())
            }
            _other => Err(NetworkBridgeErr::Other("Bad Protocol".to_string())),
        }*/
    }
}