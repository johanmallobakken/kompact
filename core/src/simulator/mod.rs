

use self::{simulation_network::SimulationNetwork, simulation_network_dispatcher::{SimulationNetworkConfig, SimulationNetworkDispatcher}};

use super::*;
use arc_swap::ArcSwap;
use executors::*;
use messaging::{DispatchEnvelope, RegistrationResult};
use prelude::{NetworkConfig, NetworkStatusPort};
use rustc_hash::FxHashMap;
pub mod simulation_network_dispatcher;
pub mod simulation_bridge;
pub mod simulation_network;
use crate::{
    runtime::*,
    timer::{
        timer_manager::TimerRefFactory,

    }, net::{ConnectionState, buffers::BufferChunk}, lookup::ActorStore, dispatch::queue_manager::QueueManager, serde_serialisers::Serde
};
use std::{
    collections::{
        HashMap,
        VecDeque
    },
    rc::Rc,
    cell::RefCell,
    future::Future,
    net::SocketAddr,
    ops::Deref,
    pin::Pin,
    sync::{
        atomic::{AtomicBool, Ordering},
        mpsc, Arc, Mutex,
    },
    task::{Context, Poll},
    time::Duration,
    thread::{
        current,
    },
    mem::drop,
};
use log::{
    debug,
    warn,
};
use crossbeam_channel::unbounded;
use async_task::*;

use std::io::{stdin,stdout,Write};

use config_keys::*;

use backtrace::Backtrace;

type SimulationNetHashMap<K, V> = FxHashMap<K, V>;

#[derive(Clone, Copy)]
pub enum SimulatedScheduling{
    Future, 
    Now, 
    Queue
}

#[derive(Clone)]
struct SimulationScheduler(Rc<RefCell<SimulationSchedulerData>>);

unsafe impl Sync for SimulationScheduler {}
unsafe impl Send for SimulationScheduler {}

pub struct SimulationSchedulerData {
    scheduling: SimulatedScheduling,
    queue: VecDeque<Arc<dyn CoreContainer>>,
}

impl SimulationSchedulerData {
    pub fn new() -> SimulationSchedulerData {
        SimulationSchedulerData {
            scheduling: SimulatedScheduling::Now, 
            queue: VecDeque::new(),
        }
    }
}

fn maybe_reschedule(c: Arc<dyn CoreContainer>) {
    match c.execute() {
        SchedulingDecision::Schedule => {
            if cfg!(feature = "use_local_executor") {
                println!("local?");
                let res = try_execute_locally(move || maybe_reschedule(c));
                assert!(!res.is_err(), "Only run with Executors that can support local execute or remove the avoid_executor_lookups feature!");
            } else {
                println!("nonlocal?");
                let c2 = c.clone();
                c.system().schedule(c2);
            }
        }
        SchedulingDecision::Resume => {
            println!("Resume");
            maybe_reschedule(c)
        },
        _ => {
            println!("Notn");
            ()
        },
    }
}

impl Scheduler for SimulationScheduler {
    fn schedule(&self, c: Arc<dyn CoreContainer>) -> () {
        println!("schedule: {:?}", std::thread::current().id());

        match current().name() {
            None => println!("SCHEDULE thread name"),
            Some(thread_name) => {
                println!("SCHEDULE Thread name: {}", thread_name)
            },
        }

        println!("schedule: {} name: {}", c.id(), c.type_name());

        match self.get_scheduling() {
            SimulatedScheduling::Future => {
                println!("FUTURE");
                maybe_reschedule(c);
                println!("AFTER FUTURE");
            },
            SimulatedScheduling::Now => {
                println!("NOW");
                c.execute();
                println!("AFTER NOW");
            },
            SimulatedScheduling::Queue => {
                println!("QUEUE");
                self.push_back_to_queue(c);
                println!("AFTER QUEUE");
            },
        }
    }

    fn shutdown_async(&self) -> (){
        println!("TODO shutdown_async");
        todo!();
    }

    fn shutdown(&self) -> Result<(), String>{
        println!("TODO shutdown");
        todo!();
    }

    fn box_clone(&self) -> Box<dyn Scheduler>{
        println!("TODO box_clone");
        Box::new(self.clone()) as Box<dyn Scheduler>
    }

    fn poison(&self) -> (){
        println!("TODO poison");
        todo!();
    }

    fn spawn(&self, future: futures::future::BoxFuture<'static, ()>) -> (){
        println!("TODO spawn");
        todo!();
    }
}

impl SimulationScheduler {
    fn get_scheduling(&self) -> SimulatedScheduling{
        self.0.as_ref().borrow().scheduling
    }

    fn push_back_to_queue(&self, c: Arc<dyn CoreContainer>){
        self.0.as_ref().borrow_mut().queue.push_back(c);
    }
}

#[derive(Clone)]
struct SimulationTimer(Rc<RefCell<SimulationTimerData>>);

unsafe impl Sync for SimulationTimer {}
unsafe impl Send for SimulationTimer {}

struct SimulationTimerData{
    inner: timer::SimulationTimer,
}

impl SimulationTimerData {
    pub(crate) fn new() -> SimulationTimerData {
        SimulationTimerData {
            inner: timer::SimulationTimer::new(),
        }
    }
}

impl TimerRefFactory for SimulationTimer {
    fn timer_ref(&self) -> timer::TimerRef {
        self.0.as_ref().borrow_mut().inner.timer_ref()
    }
}

impl TimerComponent for SimulationTimer{
    fn shutdown(&self) -> Result<(), String> {
        todo!();
    }
}

pub struct SimulationScenario {
    systems: Vec<KompactSystem>,
    scheduler: SimulationScheduler,
    timer: SimulationTimer,
    network: Arc<Mutex<SimulationNetwork>>
}

impl SimulationScenario {
    pub fn new() -> SimulationScenario {
        SimulationScenario {
            systems: Vec::new(),
            scheduler: SimulationScheduler(Rc::new(RefCell::new(SimulationSchedulerData::new()))),
            timer: SimulationTimer(Rc::new(RefCell::new(SimulationTimerData::new()))),
            //network: Rc::new(RefCell::new(SimulationNetwork::new()))
            network: Arc::new(Mutex::new(SimulationNetwork::new()))
        }
    }

    pub fn spawn_system(&mut self, cfg: KompactConfig) -> KompactSystem {
        let mut mut_cfg = cfg;
        KompactConfig::set_config_value(&mut mut_cfg, &config_keys::system::THREADS, 1usize);
        let scheduler = self.scheduler.clone();
        mut_cfg.scheduler(move |_| Box::new(scheduler.clone()));
        let timer = self.timer.clone();
        mut_cfg.timer::<SimulationTimer, _>(move || Box::new(timer.clone()));
        let dispatcher = SimulationNetworkConfig::default().build(self.network.clone());
        mut_cfg.system_components(DeadletterBox::new, dispatcher);
        let system = mut_cfg.build().expect("system");
        system
    }

    pub fn end_setup(&mut self) -> () {
        self.scheduler.0.as_ref().borrow_mut().scheduling = SimulatedScheduling::Queue;
    }
    
    pub fn break_link(&mut self, sys1: KompactSystem, sys2: KompactSystem) -> () {
        self.network.lock().unwrap().break_link(sys1.system_path(), sys2.system_path());
    }

    pub fn restore_link(&mut self, sys1: KompactSystem, sys2: KompactSystem) -> () {
        self.network.lock().unwrap().restore_link(sys1.system_path(), sys2.system_path());
    }

    pub fn shutdown_system(&self, sys: KompactSystem) -> (){
        println!("Shutting down system?");
        //self.set_scheduling_choice(SimulatedScheduling::Future);
        //sys.kill_system()
        //sys.shutdown().expect("shutdown");
        //self.set_scheduling_choice(SimulatedScheduling::Queue);
    }

    pub fn set_scheduling_choice(&self, choice: SimulatedScheduling) {
        self.scheduler.0.as_ref().borrow_mut().scheduling = choice;
    }

    fn get_work(&mut self) -> Option<Arc<dyn CoreContainer>> {
        self.scheduler.0.as_ref().borrow_mut().queue.pop_front()
    }

    pub fn create_and_register<C, F>(
        &self,
        sys: KompactSystem,
        f: F,
    ) -> (Arc<Component<C>>, KFuture<RegistrationResult>)
    where
        F: FnOnce() -> C,
        C: ComponentDefinition + 'static,
    {
        sys.create_and_register(f)
    }

    /*fn register_system_to_simulated_network(&mut self, dispatcher: SimulationNetworkDispatcher) -> () {
        self.network.lock().unwrap().register_system(dispatcher., actor_store)
    }*/

    pub fn get_simulated_network(&self) -> Arc<Mutex<SimulationNetwork>>{
        self.network.clone()
    }

    pub fn simulate_step(&mut self) -> () {
        match self.get_work(){
            Some(w) => {
                //println!("EXECUTING SOMETHING!!!!!!!!!!!!!!!!");
                let res = w.execute();
                println!("helo");
                match res {
                    SchedulingDecision::Schedule => self.scheduler.0.as_ref().borrow_mut().queue.push_back(w),
                    SchedulingDecision::AlreadyScheduled => todo!(),
                    SchedulingDecision::NoWork => (),
                    SchedulingDecision::Blocked => todo!(),
                    SchedulingDecision::Resume => todo!(),
                }
            },
            None => ()
        }
    }

    pub fn simulate_to_completion(&mut self) {
        loop {
            self.simulate_step();
        }
    }
}