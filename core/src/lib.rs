#![allow(unused_parens)]
#![feature(try_from)]
#![feature(specialization)]
#![feature(fnbox)]
#![feature(duration_as_u128)]
#![feature(drain_filter)]
//extern crate futures;
//extern crate futures_cpupool;
extern crate as_num;
extern crate bytes;
extern crate crossbeam;
extern crate executors;
extern crate num_cpus;
extern crate oncemutex;
extern crate serde;
extern crate uuid;
#[macro_use]
extern crate component_definition_derive;
#[macro_use]
extern crate actor_derive;
extern crate sequence_trie as trie;
#[macro_use]
extern crate slog;
extern crate slog_async;
extern crate slog_term;

pub use self::actors::*;
pub use self::component::*;
use self::default_components::*;
pub use self::lifecycle::*;
pub use self::ports::*;
pub use self::runtime::*;
pub use self::serialisation::*;
pub use self::timer_manager::*;
pub use self::utils::*;
pub use actor_derive::*;
pub use component_definition_derive::*;
pub use slog::{Drain, Fuse, Logger};
pub use slog_async::Async;
pub use std::convert::{From, Into};

mod actors;
mod component;
pub mod default_components;
mod default_serialisers;
mod dispatch;
mod lifecycle;
pub mod messaging;
mod ports;
mod runtime;
mod serialisation;
mod supervision;
pub mod timer;
mod timer_manager;
mod utils;

pub type KompicsLogger = Logger<std::sync::Arc<Fuse<Async>>>;

#[cfg(test)]
mod tests {

    use std::{thread, time};
    //use futures::{Future, future};
    //use futures_cpupool::CpuPool;
    use super::*;
    use bytes::Buf;
    use std::any::Any;
    use std::sync::Arc;
    use std::time::Duration;

    struct TestPort;

    impl Port for TestPort {
        type Indication = Arc<String>;
        type Request = Arc<u64>;
    }

    #[derive(ComponentDefinition, Actor)]
    struct TestComponent {
        ctx: ComponentContext<TestComponent>,
        test_port: ProvidedPort<TestPort, TestComponent>,
        counter: u64,
    }

    impl TestComponent {
        fn new() -> TestComponent {
            TestComponent {
                ctx: ComponentContext::new(),
                counter: 0,
                test_port: ProvidedPort::new(),
            }
        }
    }

    impl Provide<ControlPort> for TestComponent {
        fn handle(&mut self, event: ControlEvent) -> () {
            match event {
                ControlEvent::Start => {
                    info!(self.ctx.log(), "Starting TestComponent");
                }
                _ => (), // ignore
            }
        }
    }

    impl Provide<TestPort> for TestComponent {
        fn handle(&mut self, event: Arc<u64>) -> () {
            self.counter += *event;
            self.test_port.trigger(Arc::new(String::from("Test")));
        }
    }

    #[derive(ComponentDefinition)]
    struct RecvComponent {
        ctx: ComponentContext<RecvComponent>,
        test_port: RequiredPort<TestPort, RecvComponent>,
        last_string: String,
    }

    impl RecvComponent {
        fn new() -> RecvComponent {
            RecvComponent {
                ctx: ComponentContext::new(),
                test_port: RequiredPort::new(),
                last_string: String::from("none ;("),
            }
        }
    }

    impl Actor for RecvComponent {
        fn receive_local(&mut self, sender: ActorRef, msg: Box<Any>) -> () {
            info!(self.ctx.log(), "RecvComponent received {:?}", msg);
            if let Ok(s) = msg.downcast::<String>() {
                self.last_string = *s;
            }
            sender.tell(Box::new("Msg Received".to_string()), self);
            //sender.actor_path().tell("Msg Received", self);
        }
        fn receive_message(&mut self, sender: ActorPath, _ser_id: u64, _buf: &mut Buf) -> () {
            error!(self.ctx.log(), "Got unexpected message from {}", sender);
            unimplemented!(); // shouldn't happen during the test
        }
    }

    impl Provide<ControlPort> for RecvComponent {
        fn handle(&mut self, event: ControlEvent) -> () {
            match event {
                ControlEvent::Start => {
                    info!(self.ctx.log(), "Starting RecvComponent");
                }
                _ => (), // ignore
            }
        }
    }

    impl Require<TestPort> for RecvComponent {
        fn handle(&mut self, event: Arc<String>) -> () {
            info!(self.ctx.log(), "Got event {}", event.as_ref());
            self.last_string = event.as_ref().clone();
        }
    }

    #[test]
    fn default_settings() {
        //let pool = ThreadPool::new(2);
        let system = KompicsSystem::default();

        test_with_system(system);
    }

    #[test]
    fn custom_settings() {
        //let pool = ThreadPool::new(2);
        let mut settings = KompicsConfig::new();
        settings
            .threads(4)
            .scheduler(|t| executors::threadpool_executor::ThreadPoolExecutor::new(t));
        let system = KompicsSystem::new(settings);

        test_with_system(system);
    }

    fn test_with_system(system: KompicsSystem) -> () {
        let tc = system.create(TestComponent::new);
        let rc = system.create(RecvComponent::new);
        let rctp = rc.on_definition(|c| c.test_port.share());
        let tctp = tc.on_definition(|c| {
            c.test_port.connect(rctp);
            c.test_port.share()
        });
        system.start(&tc);
        system.start(&rc);
        let msg = Arc::new(1234);
        system.trigger_r(msg, tctp);

        let ten_millis = time::Duration::from_millis(1000);

        thread::sleep(ten_millis);

        tc.on_definition(|c| {
            //println!("Counter is {}", c.counter);
            assert_eq!(c.counter, 1234);
        });

        thread::sleep(ten_millis);

        rc.on_definition(|c| {
            //println!("Last string was {}", c.last_string);
            assert_eq!(c.last_string, String::from("Test"));
        });

        let rcref = rc.actor_ref();
        rcref.tell(Box::new(String::from("MsgTest")), &system);

        thread::sleep(ten_millis);

        rc.on_definition(|c| {
            //println!("Last string was {}", c.last_string);
            assert_eq!(c.last_string, String::from("MsgTest"));
        });

        system
            .shutdown()
            .expect("Kompics didn't shut down properly");
    }

    #[derive(ComponentDefinition, Actor)]
    struct TimerRecvComponent {
        ctx: ComponentContext<TimerRecvComponent>,
        last_string: String,
    }

    impl TimerRecvComponent {
        fn new() -> TimerRecvComponent {
            TimerRecvComponent {
                ctx: ComponentContext::new(),
                last_string: String::from("none ;("),
            }
        }
    }

    impl Provide<ControlPort> for TimerRecvComponent {
        fn handle(&mut self, event: ControlEvent) -> () {
            match event {
                ControlEvent::Start => {
                    info!(self.ctx.log(), "Starting TimerRecvComponent");
                    self.schedule_once(Duration::from_millis(100), |self_c, _| {
                        self_c.last_string = String::from("TimerTest");
                    });
                }
                _ => (), // ignore
            }
        }
    }

    #[test]
    fn test_timer() -> () {
        let system = KompicsSystem::default();
        let trc = system.create_and_start(TimerRecvComponent::new);

        thread::sleep(Duration::from_millis(1000));

        trc.on_definition(|c| {
            //println!("Counter is {}", c.counter);
            assert_eq!(c.last_string, "TimerTest");
        });

        system
            .shutdown()
            .expect("Kompics didn't shut down properly");
    }

    #[derive(ComponentDefinition)]
    struct CounterComponent {
        ctx: ComponentContext<CounterComponent>,
        msg_count: usize,
    }

    impl CounterComponent {
        fn new() -> CounterComponent {
            CounterComponent {
                ctx: ComponentContext::new(),
                msg_count: 0,
            }
        }
    }

    impl Provide<ControlPort> for CounterComponent {
        fn handle(&mut self, event: ControlEvent) -> () {
            match event {
                ControlEvent::Start => {
                    info!(self.ctx.log(), "Starting CounterComponent");
                }
                ControlEvent::Stop => {
                    info!(self.ctx.log(), "Stopping CounterComponent");
                }
                _ => (), // ignore
            }
        }
    }

    impl Actor for CounterComponent {
        fn receive_local(&mut self, _sender: ActorRef, _msg: Box<Any>) -> () {
            info!(self.ctx.log(), "CounterComponent got a message!");
            self.msg_count += 1;
        }
        fn receive_message(&mut self, sender: ActorPath, _ser_id: u64, _buf: &mut Buf) -> () {
            crit!(self.ctx.log(), "Got unexpected message from {}", sender);
            unimplemented!(); // shouldn't happen during the test
        }
    }

    #[test]
    fn test_start_stop() -> () {
        let system = KompicsSystem::default();
        let cc = system.create_and_register(CounterComponent::new);
        let ccref = cc.actor_ref();

        ccref.tell(Box::new(String::from("MsgTest")), &system);

        thread::sleep(Duration::from_millis(1000));

        cc.on_definition(|c| {
            println!("Counter is {}", c.msg_count);
            assert_eq!(c.msg_count, 0); // not yet started
        });

        let f = system.start_notify(&cc);

        f.await_timeout(Duration::from_millis(1000))
            .expect("Component never started!");

        cc.on_definition(|c| {
            println!("Counter is {}", c.msg_count);
            assert_eq!(c.msg_count, 1);
        });

        let f = system.stop_notify(&cc);
        ccref.tell(Box::new(String::from("MsgTest")), &system);

        f.await_timeout(Duration::from_millis(1000))
            .expect("Component never stopped!");

        cc.on_definition(|c| {
            println!("Counter is {}", c.msg_count);
            assert_eq!(c.msg_count, 1);
        });

        let f = system.start_notify(&cc);

        f.await_timeout(Duration::from_millis(1000))
            .expect("Component never started again!");

        cc.on_definition(|c| {
            println!("Counter is {}", c.msg_count);
            assert_eq!(c.msg_count, 2);
        });

        let f = system.kill_notify(cc);

        f.await_timeout(Duration::from_millis(1000))
            .expect("Component never died!");

        system
            .shutdown()
            .expect("Kompics didn't shut down properly");
    }
}