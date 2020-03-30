use std::{
    fmt,
    io,
    thread,
    time::{Duration, Instant, SystemTime},
};
use uuid::Uuid;

pub use self::{simulation::*, thread_timer::*, wheels::*};

mod simulation;
mod thread_timer;
mod wheels;

/// The API for Kompact's timer module
///
/// This allows behaviours to be scheduled for later execution.
pub trait Timer {
    /// Schedule the `action` to be executed once after the `timeout` expires
    ///
    /// # Note
    ///
    /// Depending on your system and the implementation used,
    /// there is always a certain lag between the execution of the `action`
    /// and the `timeout` expiring on the system's clock.
    /// Thus it is only guaranteed that the `action` is not run *before*
    /// the `timeout` expires, but no bounds on the lag are given.
    fn schedule_once<F>(&mut self, id: Uuid, timeout: Duration, action: F) -> ()
    where
        F: FnOnce(Uuid) + Send + 'static;

    /// Schedule the `action` to be run every `timeout` time units
    ///
    /// The first time, the `action` will be run after `delay` expires,
    /// and then again every `timeout` time units after.
    ///
    /// # Note
    ///
    /// Depending on your system and the implementation used,
    /// there is always a certain lag between the execution of the `action`
    /// and the `timeout` expiring on the system's clock.
    /// Thus it is only guaranteed that the `action` is not run *before*
    /// the `timeout` expires, but no bounds on the lag are given.
    fn schedule_periodic<F>(
        &mut self,
        id: Uuid,
        delay: Duration,
        period: Duration,
        action: F,
    ) -> ()
    where
        F: Fn(Uuid) -> TimerReturn + Send + 'static;

    /// Cancel the timer indicated by the `id`
    ///
    /// This method is asynchronous, and calling it is no guarantee
    /// than an already scheduled timeout is not going to fire before it
    /// is actually cancelled.
    ///
    /// However, calling this method will definitely prevent *periodic* timeouts
    /// from being rescheduled.
    fn cancel(&mut self, id: Uuid);
}

/// Indicate whether or not to reschedule a periodic timer
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TimerReturn {
    /// Reschedule the timer
    Reschedule,
    /// Do not reschedule the timer
    Cancel,
}
impl TimerReturn {
    /// Whether or not this timer should be rescheduled
    pub fn should_reschedule(&self) -> bool {
        match self {
            TimerReturn::Reschedule => true,
            TimerReturn::Cancel => false,
        }
    }
}
impl Default for TimerReturn {
    fn default() -> Self {
        TimerReturn::Reschedule
    }
}

/// A concrete entry for an outstanding timout
pub enum TimerEntry {
    /// A one-off timer
    OneShot {
        /// The unique of the timeout
        id: Uuid,
        /// The length of the timeout
        timeout: Duration,
        /// The action to invoke when the timeout expires
        action: Box<dyn FnOnce(Uuid) + Send + 'static>,
    },
    /// A recurring timer
    Periodic {
        /// The unique of the timeout
        id: Uuid,
        /// The delay until the `action` is first invoked
        delay: Duration,
        /// The time between `action` invocations
        period: Duration,
        /// The action to invoke whenever the timeout expires
        action: Box<dyn Fn(Uuid) -> TimerReturn + Send + 'static>,
    },
}

impl TimerEntry {
    /// Returns the unique id of the outstanding timeout
    pub fn id(&self) -> Uuid {
        match self {
            TimerEntry::OneShot { id, .. } => *id,
            TimerEntry::Periodic { id, .. } => *id,
        }
    }

    /// Returns a reference to the unique id of the outstanding timeout
    pub fn id_ref(&self) -> &Uuid {
        match self {
            TimerEntry::OneShot { id, .. } => id,
            TimerEntry::Periodic { id, .. } => id,
        }
    }

    /// Returns the time until the timeout is supposed to be triggered
    pub fn delay(&self) -> Duration {
        match self {
            TimerEntry::OneShot { timeout, .. } => *timeout,
            TimerEntry::Periodic { delay, .. } => *delay,
        }
    }

    /// Returns a reference to the time until the timeout is supposed to be triggered
    pub fn delay_ref(&self) -> &Duration {
        match self {
            TimerEntry::OneShot { timeout, .. } => timeout,
            TimerEntry::Periodic { delay, .. } => delay,
        }
    }

    /// Updates the `timeout` or `delay` value of the entry with the given duration `d`
    pub fn with_duration(self, d: Duration) -> TimerEntry {
        match self {
            TimerEntry::OneShot { id, action, .. } => TimerEntry::OneShot {
                id,
                timeout: d,
                action,
            },
            TimerEntry::Periodic {
                id, period, action, ..
            } => TimerEntry::Periodic {
                id,
                delay: d,
                period,
                action,
            },
        }
    }

    /// Execute the action associated with the timeout
    ///
    /// Returns a new timer entry, if it needs to be rescheduled
    /// and `None` otherwise.
    pub fn execute(self) -> Option<TimerEntry> {
        match self {
            TimerEntry::OneShot { id, action, .. } => {
                action(id);
                None
            }
            TimerEntry::Periodic {
                id, action, period, ..
            } => {
                let res = action.as_ref()(id);
                if res.should_reschedule() {
                    let next = TimerEntry::Periodic {
                        id,
                        delay: period,
                        period,
                        action,
                    };
                    Some(next)
                } else {
                    None
                }
            }
        }
    }
}

impl fmt::Debug for TimerEntry {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TimerEntry::OneShot { id, timeout, .. } => write!(
                f,
                "TimerEntry::OneShot(id={:?}, timeout={:?}, action=<function>)",
                id, timeout
            ),
            TimerEntry::Periodic {
                id, delay, period, ..
            } => write!(
                f,
                "TimerEntry::Periodic(id={:?}, delay={:?}, period={:?} action=<function>)",
                id, delay, period
            ),
        }
    }
}

type TimerList = Vec<TimerEntry>;

/// Errors encounted by a timer implementation
#[derive(Debug)]
pub enum TimerError {
    /// The timeout with the given id was not found
    NotFound,
    /// The timout has already expired
    Expired(TimerEntry),
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{Arc, Mutex};

    #[test]
    fn simple_simulation() {
        let num = 10usize;
        let mut barriers: Vec<Arc<Mutex<bool>>> = Vec::with_capacity(num);
        let mut timer = SimulationTimer::new();
        for i in 0..num {
            let barrier = Arc::new(Mutex::new(false));
            barriers.push(barrier.clone());
            let id = Uuid::new_v4();
            let timeout = Duration::from_millis(150u64 * (i as u64));
            timer.schedule_once(id, timeout, move |_| {
                println!("Running action {}", i);
                let mut guard = barrier.lock().unwrap();
                *guard = true;
            });
        }
        let mut running = true;
        while running {
            match timer.next() {
                SimulationStep::Ok => println!("Next!"),
                SimulationStep::Finished => running = false,
            }
        }
        println!("Simulation run done!");
        for b in barriers {
            let guard = b.lock().unwrap();
            assert_eq!(*guard, true);
        }
    }

    // TODO test with rescheduling

    fn fib_time(mut i: usize) -> Duration {
        if i == 0 {
            Duration::from_millis(0)
        } else if i == 1 {
            Duration::from_millis(1)
        } else {
            let mut fminus2: u64 = 0u64;
            let mut fminus1: u64 = 1u64;
            while i >= 2 {
                let fi = fminus2 + fminus1;
                i -= 1;
                fminus2 = fminus1;
                fminus1 = fi;
            }
            Duration::from_millis(fminus1)
        }
    }

    #[test]
    fn simple_thread_timing() {
        let num = 20usize;
        let mut barriers: Vec<Arc<Mutex<bool>>> = Vec::with_capacity(num);
        let timer_core = TimerWithThread::new().expect("Timer thread didn't load properly!");
        let mut timer = timer_core.timer_ref();
        let mut total_wait = Duration::from_millis(0);
        println!("Starting timing run.");
        for i in 0..num {
            let barrier = Arc::new(Mutex::new(false));
            barriers.push(barrier.clone());
            let id = Uuid::new_v4();
            let timeout = fib_time(i);
            total_wait += timeout;
            let now = Instant::now();
            timer.schedule_once(id, timeout, move |_| {
                let elap = now.elapsed().as_nanos();
                let target = timeout.as_nanos();
                if (elap > target) {
                    let diff = ((elap - target) as f64) / 1000000.0;
                    println!("Running action {} {}ms late", i, diff);
                } else {
                    let diff = ((target - elap) as f64) / 1000000.0;
                    println!("Running action {} {}ms early", i, diff);
                }
                let mut guard = barrier.lock().unwrap();
                *guard = true;
            });
        }
        println!("Waiting timing run to finish {}ms", total_wait.as_millis());
        thread::sleep(total_wait);
        timer_core
            .shutdown()
            .expect("Timer didn't shutdown properly!");
        println!("Timing run done!");
        for b in barriers {
            let guard = b.lock().unwrap();
            assert_eq!(*guard, true);
        }
    }
}
