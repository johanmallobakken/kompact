use self::{simulation_network::SimulationNetwork, simulation_network_dispatcher::{SimulationNetworkConfig, SimulationNetworkDispatcher}};

use super::*;
use arc_swap::ArcSwap;
use executors::*;
use hierarchical_hash_wheel_timer::{Timer, simulation::SimulationStep};
use messaging::{DispatchEnvelope, RegistrationResult};
use prelude::{NetworkConfig, NetworkStatusPort};
use rustc_hash::FxHashMap;
use std::{fmt::{Debug, Display}, fs::{self, File}};
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
            scheduling: SimulatedScheduling::Queue, 
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
                println!("Scheduled again");
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
        /*let res = c.execute();
        let add_to_queue = match res {
            SchedulingDecision::NoWork | SchedulingDecision::Blocked => true,
            SchedulingDecision::Resume | SchedulingDecision::Schedule => false,
            SchedulingDecision::AlreadyScheduled => {
                panic!("Don't know what to do with AlreadyScheduled here")
            }
        };
        if add_to_queue {
        }*/

        /*println!("SOMETHING SCHEDULED");
        if self.get_queue_len() > 0 {
            println!("Something in queue, len {}", self.get_queue_len());
            match self.pop_from_queue() {
                Some(scheduled_component) => {
                    maybe_reschedule(scheduled_component);
                },
                None => panic!("Should be a component scheduled"),
            }
        } else {
            println!("Nothing in queue");
            /*let res = c.execute();
            let add_to_queue = match res {
                SchedulingDecision::NoWork | SchedulingDecision::Blocked => false,
                SchedulingDecision::Resume | SchedulingDecision::Schedule => true,
                SchedulingDecision::AlreadyScheduled => {
                    panic!("Don't know what to do with AlreadyScheduled here")
                }
            };

            if add_to_queue {
                let c2 = c.clone();
                c.system().schedule(c2);
            }*/

            self.push_back_to_queue(c)
        }*/
        /*let res = c.execute();
        let add_to_queue = match res {
            SchedulingDecision::NoWork | SchedulingDecision::Blocked => false,
            SchedulingDecision::Resume | SchedulingDecision::Schedule => true,
            SchedulingDecision::AlreadyScheduled => {
                panic!("Don't know what to do with AlreadyScheduled here")
            }
        };
        if add_to_queue {
            let c2 = c.clone();
            self.schedule(c2)
        }*/

        match self.get_scheduling() {
            SimulatedScheduling::Future => {
                maybe_reschedule(c);
            },
            SimulatedScheduling::Now => {
                match c.execute() {
                    SchedulingDecision::Schedule | SchedulingDecision::Resume  => {
                        let c2 = c.clone();
                        c.system().schedule(c2);
                    },
                    SchedulingDecision::NoWork | SchedulingDecision::Blocked => (),
                    SchedulingDecision::AlreadyScheduled => panic!("Already Scheduled"),
                }
            },
            SimulatedScheduling::Queue => {
                self.push_back_to_queue(c);
            },
        }
    }

    fn shutdown_async(&self) -> (){
        //todo!();
        println!("shutdown_async");
    }

    fn shutdown(&self) -> Result<(), String>{
        println!("shutdown");
        Ok(())
    }

    fn box_clone(&self) -> Box<dyn Scheduler>{
        Box::new(self.clone()) as Box<dyn Scheduler>
    }

    fn poison(&self) -> (){
        //todo!();
        println!("poison")
    }

    fn spawn(&self, future: futures::future::BoxFuture<'static, ()>) -> (){
        //todo!();
        println!("spawn")
    }
}

impl SimulationScheduler {
    fn get_scheduling(&self) -> SimulatedScheduling{
        self.0.as_ref().borrow().scheduling
    }

    fn push_back_to_queue(&self, c: Arc<dyn CoreContainer>){
        self.0.as_ref().borrow_mut().queue.push_back(c);
    }

    fn pop_from_queue(&self) -> Option<Arc<dyn CoreContainer>>{
        self.0.as_ref().borrow_mut().queue.pop_front()
    }

    fn get_queue_len(&self) -> usize{
        self.0.as_ref().borrow().queue.len()
    }
}

#[derive(Clone)]
struct SimulationTimer(Rc<RefCell<SimulationTimerData>>);

unsafe impl Sync for SimulationTimer {}
unsafe impl Send for SimulationTimer {}

struct SimulationTimerData{
    pub inner: timer::SimulationTimer,
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

struct SimualtionState<T>
where T: 'static
{
    state: T
}

impl<T> SimualtionState<T>{
    pub fn new(state: T){

    }
}

pub struct SimulationScenario<T>{
    systems: Vec<KompactSystem>,
    scheduler: SimulationScheduler,
    timer: SimulationTimer,
    network: Arc<Mutex<SimulationNetwork>>,
    monitored_invariants: Vec<Arc<dyn Invariant<T>>>,
    monitored_actors: Vec<Arc<dyn GetState<T>>>,
    simulation_step_count: u64,
    state_file: File,
    prev_state_string: String
}

impl<T: Debug + Display + 'static> SimulationScenario<T>{
    pub fn new() -> SimulationScenario<T>{
        SimulationScenario {
            systems: Vec::new(),
            scheduler: SimulationScheduler(Rc::new(RefCell::new(SimulationSchedulerData::new()))),
            timer: SimulationTimer(Rc::new(RefCell::new(SimulationTimerData::new()))),
            //network: Rc::new(RefCell::new(SimulationNetwork::new()))
            network: Arc::new(Mutex::new(SimulationNetwork::new())),
            monitored_actors: Vec::new(),
            monitored_invariants: Vec::new(),
            simulation_step_count: 0,
            state_file: File::create("state.txt").unwrap(),
            prev_state_string: String::new()
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
        self.set_scheduling_choice(SimulatedScheduling::Now);
        let mut mut_cfg = cfg;
        KompactConfig::set_config_value(&mut mut_cfg, &config_keys::system::THREADS, 1usize);
        let scheduler = self.scheduler.clone();
        mut_cfg.scheduler(move |_| Box::new(scheduler.clone()));
        let timer = self.timer.clone();
        mut_cfg.timer::<SimulationTimer, _>(move || Box::new(timer.clone()));
        let dispatcher = SimulationNetworkConfig::default().build(self.network.clone());
        mut_cfg.system_components(DeadletterBox::new, dispatcher);
        let system = mut_cfg.build().expect("system");
        self.set_scheduling_choice(SimulatedScheduling::Queue);
        self.systems.push(system.clone());
        system
    }

    pub fn create_and_register<C, F>(
        &mut self,
        sys: &KompactSystem,
        f: F,
        register_timeout: Duration
    ) -> (Arc<Component<C>>, ActorPath)
    where
        F: FnOnce() -> C,
        C: ComponentDefinition + 'static,
    {
        self.set_scheduling_choice(SimulatedScheduling::Now);
        let (component, registration_future) = sys.create_and_register(f);

        //self.monitor_actor(actor)
        let path = registration_future.wait_expect(register_timeout, "Failed to Register create_and_register");
        self.set_scheduling_choice(SimulatedScheduling::Queue);
        (component, path)
    }

    pub fn register_by_alias<A>(
        &mut self,
        sys: &KompactSystem,
        c: &dyn DynActorRefFactory,
        alias: A,
        register_timeout: Duration
    ) -> ActorPath
    where
        A: Into<String>,
    {
        /*self.inner.assert_active();
        self.inner
            .register_by_alias(c.dyn_ref(), false, alias.into())*/
        self.set_scheduling_choice(SimulatedScheduling::Now);
        let path = sys.register_by_alias(c, alias).wait_expect(register_timeout, "Failed to Register register_by_alias");
        self.set_scheduling_choice(SimulatedScheduling::Queue);
        path
    }
    
    pub fn start_notify(
        &mut self, 
        sys: &KompactSystem, 
        c: &Arc<impl AbstractComponent + ?Sized>, 
        register_timeout: Duration
    ) -> () {
        self.set_scheduling_choice(SimulatedScheduling::Now);
        let raft_comp_f = sys.start_notify(c);
        raft_comp_f
            .wait_timeout(register_timeout)
            .expect("Failed start_notify");
        self.set_scheduling_choice(SimulatedScheduling::Queue);
    }

    pub fn wait_future<K>(
        &mut self, 
        future: KFuture<K>
    ) -> K 
    where
        K: Send + Sized
    {
        self.set_scheduling_choice(SimulatedScheduling::Now);

        while self.scheduler.0.as_ref().borrow().queue.len() > 0 {
            self.simulate_step();
        }

        let res = future.wait();
        self.set_scheduling_choice(SimulatedScheduling::Queue);
        res
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

    pub fn clog_system(&mut self, sys: KompactSystem) -> () {
        self.network.lock().unwrap().clog(sys.system_path().socket_address());
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

    fn next_timer(&mut self) -> SimulationStep{
        self.timer.0.as_ref().borrow_mut().inner.next()
    }

    /*fn register_system_to_simulated_network(&mut self, dispatcher: SimulationNetworkDispatcher) -> () {
        self.network.lock().unwrap().register_system(dispatcher., actor_store)
    }*/

    pub fn get_simulated_network(&self) -> Arc<Mutex<SimulationNetwork>>{
        self.network.clone()
    }

    fn print_all_actor_states(&self) {
        for actor in &self.monitored_actors{
            println!("{:?}", actor.get_state());
        }
    }

    fn write_states_to_file(&mut self) {

        let mut actor_states_string = String::new();

        for actor in &self.monitored_actors {
            actor_states_string.push_str(&actor.get_state().to_string());
            actor_states_string.push_str("\n");
        }

        if actor_states_string != self.prev_state_string {
            let mut step_count_string = self.simulation_step_count.to_string();
            step_count_string.push_str("\n");

            self.state_file.write(step_count_string.as_bytes()).expect("Unable to write file");
            self.state_file.write(actor_states_string.as_bytes()).expect("Unable to write file");
        }
    }


    pub fn simulate_step(&mut self) -> () {

        //println!("Step ID: {}", self.simulation_step_count);
        //self.print_all_actor_states();
        self.write_states_to_file();
        self.simulation_step_count += 1;

        self.next_timer();
        
        match self.get_work(){
            Some(w) => {
                let res = w.execute();
                match res {
                    SchedulingDecision::Schedule | SchedulingDecision::Resume => self.scheduler.schedule(w), //self.scheduler.0.as_ref().borrow_mut().queue.push_back(w),                    
                    SchedulingDecision::NoWork | SchedulingDecision::Blocked => (),
                    SchedulingDecision::AlreadyScheduled => panic!("Already Scheduled"),
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