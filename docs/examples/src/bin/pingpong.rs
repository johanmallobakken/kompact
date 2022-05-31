use kompact::prelude::*;
use kompact::{prelude::*, serde_serialisers::*};
use serde::{Deserialize, Serialize};
use std::borrow::BorrowMut;
use std::cell::RefCell;
use std::rc::Rc;
use std::sync::Mutex;
use std::{
    time::Duration,
    thread::{
        current,
    }
};

#[derive(ComponentDefinition)]
struct Pinger {
    ctx: ComponentContext<Self>,
    actor_path: ActorPath,
    counter: u64
}

#[derive(ComponentDefinition)]
struct Ponger {
    lol: ComponentContext<Self>,
    counter: u64
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy)]
struct Ping;
impl SerialisationId for Ping {
    const SER_ID: SerId = 1234;
}
#[derive(Serialize, Deserialize, Debug, Clone, Copy)]
struct Pong;
impl SerialisationId for Pong {
    const SER_ID: SerId = 4321;
}

struct PingPongPort;
impl Port for PingPongPort {
    type Indication = Pong;
    type Request = Ping;
}

impl Pinger {
    pub fn new(actor_path:ActorPath) -> Self {
        Pinger {
            ctx: ComponentContext::uninitialised(),
            actor_path: actor_path,
            counter: 0
        }
    }
}

impl Ponger {
    pub fn new() -> Self {
        Ponger {
            lol: ComponentContext::uninitialised(),
            counter: 0
        }
    }
}

impl ComponentLifecycle for Pinger {
    fn on_start(&mut self) -> Handled {
        info!(self.log(), "Pinger started!");
        self.actor_path.tell((Ping, Serde), self);
        Handled::Ok
    }
}


impl ComponentLifecycle for Ponger {
    fn on_start(&mut self) -> Handled {
        info!(self.log(), "Ponger started!");
        Handled::Ok
    }
}

enum GlobalState {
    Pinger { count: i32},
    Ponger { count: i32},
}

impl Actor for Pinger {
    type Message = Never;

    fn receive_local(&mut self, msg: Self::Message) -> Handled {
        unimplemented!("We are ignoring local messages");
    }

    fn receive_network(&mut self, msg: NetMessage) -> Handled {
        let sender = msg.sender;
        match msg.data.try_deserialise::<Pong, Serde>() {
            Ok(_ping) => {
                self.counter += 1;
                println!("!!! RECIEVED PONG {} IN PINGER !!!", self.counter);
                //info!(self.log(), "RECIEVED PONG {} IN PINGER", self.counter);
                sender.tell((Ping, Serde), self)
            }
            Err(e) => warn!(self.log(), "Invalid data: {:?}", e),
        }
        Handled::Ok
    }
}

impl Actor for Ponger {
    type Message = Never;

    fn receive_local(&mut self, msg: Self::Message) -> Handled {
        unimplemented!("We are ignoring local messages");
    }

    fn receive_network(&mut self, msg: NetMessage) -> Handled {

        let sender = msg.sender;
        match msg.data.try_deserialise::<Ping, Serde>() {
            Ok(_ping) => {
                self.counter += 1;
                println!("!!! RECIEVED PING {} IN PONGER !!!", self.counter);
                info!(self.log(), "RECIEVED PING {} IN PONGER", self.counter);
                sender.tell((Pong, Serde), self)
            }
            Err(e) => warn!(self.log(), "Invalid data: {:?}", e),
        }
        Handled::Ok
    }
}

impl GetState<GlobalState> for Pinger {
    fn get_state(&self) -> GlobalState {
        todo!();
    }
}


impl GetState<GlobalState> for Ponger {
    fn get_state(&self) -> GlobalState {
        todo!();
    }
}

pub fn sim() {
    let mut simulation: SimulationScenario<GlobalState> = SimulationScenario::new();

    let mut cfg2 = KompactConfig::default();
    let sys2 = simulation.spawn_system(cfg2);

    let mut cfg1 = KompactConfig::default();
    let sys1 = simulation.spawn_system(cfg1);

    let (ponger, registration_future) = sys2.create_and_register(Ponger::new);
    let path = registration_future.wait_expect(Duration::from_millis(1000), "actor never registered");

    let (pinger, registration_future) = sys1.create_and_register(move || Pinger::new(path));
    registration_future.wait_expect(Duration::from_millis(1000), "actor never registered");

    simulation.end_setup();
    
    sys2.start(&ponger);
    println!("start in main");
    sys1.start(&pinger);
    println!("start in main 2");
    
    println!("start in main 1 {:?}", simulation.simulate_step());
    println!("start in main 2 {:?}", simulation.simulate_step());
    println!("start in main 3 {:?}", simulation.simulate_step());
    println!("start in main 4 {:?}", simulation.simulate_step());
    println!("start in main 5 {:?}", simulation.simulate_step());
    println!("start in main 6 {:?}", simulation.simulate_step());
    println!("start in main 7 {:?}", simulation.simulate_step());
    println!("start in main 8 {:?}", simulation.simulate_step());
    println!("start in main 9 {:?}", simulation.simulate_step());
    println!("start in main 10 {:?}", simulation.simulate_step());
    println!("start in main 11 {:?}", simulation.simulate_step());
    println!("start in main 12 {:?}", simulation.simulate_step());
    println!("start in main 13 {:?}", simulation.simulate_step());
    println!("start in main 14 {:?}", simulation.simulate_step());

    simulation.break_link(sys1.clone(), sys2.clone());

    println!("start in main 15 {:?}", simulation.simulate_step());
    println!("start in main 16 {:?}", simulation.simulate_step());
    println!("start in main 17 {:?}", simulation.simulate_step());

    simulation.restore_link(sys1.clone(), sys2.clone());


    println!("start in main 18 {:?}", simulation.simulate_step());
    println!("start in main 19 {:?}", simulation.simulate_step());
    println!("start in main 20 {:?}", simulation.simulate_step());
    println!("start in main 21 {:?}", simulation.simulate_step());
    println!("start in main 22 {:?}", simulation.simulate_step());
    println!("start in main 23 {:?}", simulation.simulate_step());
    println!("start in main 24 {:?}", simulation.simulate_step());
    println!("start in main 25 {:?}", simulation.simulate_step());
    println!("start in main 26 {:?}", simulation.simulate_step());
    println!("start in main 27 {:?}", simulation.simulate_step());
    println!("start in main 28 {:?}", simulation.simulate_step());
    println!("start in main 29 {:?}", simulation.simulate_step());
    println!("start in main 30 {:?}", simulation.simulate_step());
    println!("start in main 31 {:?}", simulation.simulate_step());
    println!("start in main 32 {:?}", simulation.simulate_step());

    println!("done simulation steps");

    //simulation.simulate_to_completion();
    println!("THREAD ID main pp: {:?}", std::thread::current().id());

    //std::thread::sleep(core::time::Duration::from_millis(1000));

    //simulation.set_scheduling_choice(SimulatedScheduling::Now);

    println!("FUTURE EXECUTOR FROM NOW");
    
    simulation.shutdown_system(sys1);
    //sys1.shutdown_async();
    println!("shutdown");

    simulation.shutdown_system(sys2);
    //sys2.shutdown_async();
    println!("shutdown 2");
} 



pub fn nonsim() {
    let mut cfg2 = KompactConfig::default();
    cfg2.system_components(DeadletterBox::new, NetworkConfig::default().build());
    let sys2 = cfg2.build().expect("sys2");

    let mut cfg1 = KompactConfig::default();
    cfg1.system_components(DeadletterBox::new, NetworkConfig::default().build());
    let sys1 = cfg1.build().expect("sys1");

    let (ponger, registration_future) = sys2.create_and_register(Ponger::new);
    let path = registration_future.wait_expect(Duration::from_millis(1000), "actor never registered");

    let (pinger, registration_future) = sys1.create_and_register(move || Pinger::new(path));
    registration_future.wait_expect(Duration::from_millis(1000), "actor never registered");
    
    sys2.start(&ponger);
    sys1.start(&pinger);

    std::thread::sleep(Duration::from_millis(1000));
    sys1.shutdown().expect("shutdown");
    sys2.shutdown().expect("shutdown");
}

pub fn main(){
    sim();
}
