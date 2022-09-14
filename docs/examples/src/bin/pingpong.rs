use kompact::prelude::*;
use kompact::{prelude::*, serde_serialisers::*};
use serde::{Deserialize, Serialize};
use std::borrow::BorrowMut;
use std::cell::RefCell;
use std::cmp::{min, max};
use std::fmt::Display;
use std::rc::Rc;
use std::sync::{Mutex, Arc};
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
    counter: u64,
    timer: Option<ScheduledTimer>,
}

#[derive(ComponentDefinition)]
struct Ponger {
    ctx: ComponentContext<Self>,
    counter: u64,
    timer: Option<ScheduledTimer>
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
            counter: 0,
            timer: None,
        }
    }
}

impl Ponger {
    pub fn new() -> Self {
        Ponger {
            ctx: ComponentContext::uninitialised(),
            counter: 0,
            timer: None
        }
    }
}

impl ComponentLifecycle for Pinger {
    fn on_start(&mut self) -> Handled {
        //self.actor_path.tell((Ping, Serde), self);
        self.start_timer();
        Handled::Ok
    }
}
impl ComponentLifecycle for Ponger {}

#[derive(Clone, Debug)]
struct PingPongState {
    count: u64
}

impl Display for PingPongState{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
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
                println!("received PONG in PINGER {}", self.counter);
                //sender.tell((Ping, Serde), self)
            }
            Err(e) => warn!(self.log(), "Invalid data: {:?}", e),
        }
        Handled::Ok
    }
}

impl Pinger {
    fn start_timer(&mut self){
        let ready_timer = self.schedule_periodic(Duration::from_millis(0), Duration::from_millis(10), move |c, _| {
            c.on_ready()
        });

        self.timer = Some(ready_timer);
    }

    fn on_ready(&mut self) -> Handled{
        println!("onready! Sending PING to PONGER");
        self.actor_path.tell((Ping, Serde), self);
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
                println!("received PING in PONGER {}", self.counter);
                sender.tell((Pong, Serde), self)
            }
            Err(e) => warn!(self.log(), "Invalid data: {:?}", e),
        }
        Handled::Ok
    }
}

impl GetState<PingPongState> for Component<Pinger> {
    fn get_state(&self) -> PingPongState {
        let def = &self.mutable_core.lock().unwrap().definition;
        PingPongState {
            count: def.counter
        }
    }
}
impl GetState<PingPongState> for Component<Ponger>  {
    fn get_state(&self) -> PingPongState {
        let def = &self.mutable_core.lock().unwrap().definition;
        PingPongState {
            count: def.counter
        }
    }
}

struct BalancedCountInvariant {}
impl Invariant<PingPongState> for BalancedCountInvariant {
    fn check(&self, states: Vec<PingPongState>) -> Result<(), SimulationError> {
        let min = states.iter().map(|s| s.count).min().unwrap();
        let max = states.iter().map(|s| s.count).max().unwrap();
        if max-min > 1 {
            return Err(SimulationError{message: "Count unbalanced".to_string()});
        }
        Ok(())
    }
}


pub fn sim() {
    let mut simulation: SimulationScenario<PingPongState> =
    SimulationScenario::new();

    let sys2 = simulation.spawn_system(KompactConfig::default());
    let sys1 = simulation.spawn_system(KompactConfig::default());

    let (ponger, path) = simulation.schedule_now(|| {
        let (ponger, registration_future) = sys2.create_and_register(Ponger::new);

        let path = registration_future.wait_expect(
            Duration::from_millis(1000), 
            "actor never registered"
        );
        (ponger, path)
    });

    let pinger = simulation.schedule_now(|| {
        let (pinger, registration_future) = 
        sys1.create_and_register(move || Pinger::new(path));

        registration_future.wait_expect(
            Duration::from_millis(1000), 
            "actor never registered"
        );
        pinger
    });

    simulation.monitor_component(ponger.clone());
    simulation.monitor_component(pinger.clone());

    simulation.monitor_invariant(Arc::new(BalancedCountInvariant {}));
    
    sys2.start(&ponger);
    sys1.start(&pinger);

    for _ in 0..50{
        simulation.simulate_step();
    }

    simulation.break_link(&sys2, &sys1);

    for _ in 0..50{
        simulation.simulate_step();
    }
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
    //nonsim();
    sim();
}
