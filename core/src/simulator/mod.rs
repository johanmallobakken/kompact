use self::{simulation_network::SimulationNetwork, simulation_network_dispatcher::{SimulationNetworkConfig, SimulationNetworkDispatcher}};

use super::*;
use arc_swap::ArcSwap;
use executors::*;
use hierarchical_hash_wheel_timer::Timer;
use messaging::{DispatchEnvelope, RegistrationResult};
use prelude::{NetworkConfig, NetworkStatusPort};
use rustc_hash::FxHashMap;
pub mod simulation_network_dispatcher;
pub mod simulation_network;
pub mod state;

use crate::{
    runtime::*,
    timer::{
        timer_manager::TimerRefFactory,
    }, 
    net::{ConnectionState, buffers::BufferChunk}, 
    lookup::ActorStore, 
    dispatch::queue_manager::QueueManager, 
    serde_serialisers::Serde,
};

pub use state::GetState;

use state::Invariant;
use state::ProgressCounter;
use state::SimulationError;

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

use lazy_static::lazy_static;

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
                //println!("local?");
                let res = try_execute_locally(move || maybe_reschedule(c));
                assert!(!res.is_err(), "Only run with Executors that can support local execute or remove the avoid_executor_lookups feature!");
            } else {
                //println!("nonlocal?");
                let c2 = c.clone();
                c.system().schedule(c2);
            }
        }
        SchedulingDecision::Resume => {
            //println!("Resume");
            maybe_reschedule(c)
        },
        _ => {
            //println!("Notn");
            ()
        },
    }
}

impl Scheduler for SimulationScheduler {
    fn schedule(&self, c: Arc<dyn CoreContainer>) -> () {
        /*println!("schedule: {:?}", std::thread::current().id());

        match current().name() {
            None => println!("SCHEDULE thread name"),
            Some(thread_name) => {
                println!("SCHEDULE Thread name: {}", thread_name)
            },
        }

        println!("schedule: {} name: {}", c.id(), c.type_name());*/

        match self.get_scheduling() {
            SimulatedScheduling::Future => {
                //println!("FUTURE");
                maybe_reschedule(c);
                //println!("AFTER FUTURE");
            },
            SimulatedScheduling::Now => {
                //println!("NOW");
                //println!("cid {}", c.id());
                c.execute();
                //println!("AFTER NOW");
            },
            SimulatedScheduling::Queue => {
                //println!("QUEUE");
                self.push_back_to_queue(c);
                //println!("AFTER QUEUE");
            },
        }
    }

    fn shutdown_async(&self) -> (){
        todo!();
    }

    fn shutdown(&self) -> Result<(), String>{
        todo!();
    }

    fn box_clone(&self) -> Box<dyn Scheduler>{
        Box::new(self.clone()) as Box<dyn Scheduler>
    }

    fn poison(&self) -> (){
        todo!();
    }

    fn spawn(&self, future: futures::future::BoxFuture<'static, ()>) -> (){
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
        //println!("Shutdown timer");
        Ok(())
    }
}

/* 
lazy_static! {
    static ref GLOBAL_STATE: Option<GlobalState<T>>  = Mutex::new(0);
}*/

pub struct SimulationScenario<T>{
    systems: Vec<KompactSystem>,
    scheduler: SimulationScheduler,
    timer: SimulationTimer,
    network: Arc<Mutex<SimulationNetwork>>,
    monitored_invariants: Vec<Arc<dyn Invariant<T>>>,
    monitored_actors: Vec<Arc<dyn GetState<T>>>
}

impl<T: 'static> SimulationScenario<T>{
    pub fn new() -> SimulationScenario<T>{
        SimulationScenario {
            systems: Vec::new(),
            scheduler: SimulationScheduler(Rc::new(RefCell::new(SimulationSchedulerData::new()))),
            timer: SimulationTimer(Rc::new(RefCell::new(SimulationTimerData::new()))),
            //network: Rc::new(RefCell::new(SimulationNetwork::new()))
            network: Arc::new(Mutex::new(SimulationNetwork::new())),
            monitored_actors: Vec::new(),
            monitored_invariants: Vec::new(),
        }
    }

    fn check_invariance(&self) -> Result<(), SimulationError>{
        for invariant in &self.monitored_invariants {
            let t_vec: Vec<T> = self.monitored_actors.iter().map(|actor| actor.get_state()).collect();
            invariant.check(t_vec)?;
        }
        Ok(())
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
        self.network.lock().unwrap().clog_link(sys1.system_path().socket_address(), sys2.system_path().socket_address());
    }

    pub fn restore_link(&mut self, sys1: KompactSystem, sys2: KompactSystem) -> () {
        self.network.lock().unwrap().unclog_link(sys1.system_path().socket_address(), sys2.system_path().socket_address());
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
                //println!("helo");
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

    //State related


    pub fn monitor_invariant(&mut self, invariant: Arc<dyn Invariant<T>>) {
        self.monitored_invariants.push(invariant);
    }
    pub fn monitor_actor(&mut self, actor: Arc<dyn GetState<T>>) {
        self.monitored_actors.push(actor);
    }
        /* 
    pub fn monitor_progress(&mut self, actor: &Arc<dyn ProgressCounter>) {
        // appends it to some struct? 
        todo!()
    }

    fn check_invariants(&self) -> Result<(), SimulationError> {
        // checks all invariants
        todo!()
    }
    fn check_progress(&self) -> Result<(), SimulationError> {
        // checks all progress somehow
        todo!()
    }
    pub fn run_simulation_step(&mut self) -> Result<(), SimulationError> {
        //self.check_invariants()?;
        //self.check_progress()?;
        todo!()
    }
    */
}