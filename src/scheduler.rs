
use std::collections::BinaryHeap;
use std::sync::{Condvar, Mutex};
use std::thread;
use std::sync::{Arc};
use std::sync::mpsc::{Sender, Receiver, channel};
use std::cmp::{Ordering, min, max};

use time::{SteadyTime, Duration};

use pulse::{Signal, Pulse};

struct ScheduledEvent {
    when: SteadyTime,
    completion_sink: Pulse,
}
impl Ord for ScheduledEvent {
    fn cmp(&self, other: &ScheduledEvent) -> Ordering {
        other.when.cmp(&self.when)
    }
}
impl PartialEq for ScheduledEvent {
    fn eq(&self, other: &ScheduledEvent) -> bool {
        let self_ptr: *const ScheduledEvent = self;
        let other_ptr: *const ScheduledEvent = other;

        self_ptr == other_ptr
    }
}
impl Eq for ScheduledEvent {}
impl PartialOrd for ScheduledEvent {
    fn partial_cmp(&self, other: &ScheduledEvent) -> Option<Ordering> {
        other.when.partial_cmp(&self.when)
    }
}

struct SchedulingRequest {
    duration: u32,
    completion_sink: Pulse,
}

struct SchedulingInterface {
    trigger: Arc<Condvar>,
    adder: Sender<SchedulingRequest>,
}

struct ScheduleWorker {
    trigger: Arc<Condvar>,
    request_source: Receiver<SchedulingRequest>,
    schedule: BinaryHeap<ScheduledEvent>,
}

impl ScheduleWorker {
    fn new(trigger: Arc<Condvar>, request_source: Receiver<SchedulingRequest>) -> ScheduleWorker {
        ScheduleWorker{
            trigger: trigger,
            request_source: request_source,
            schedule: BinaryHeap::new(),
        }
    }

    fn drain_request_queue(&mut self) {
        while let Ok(request) = self.request_source.try_recv() {
            self.schedule.push(ScheduledEvent{
                when: SteadyTime::now() + Duration::milliseconds(request.duration as i64),
                completion_sink: request.completion_sink
            });
        }
    }

    fn has_event_now(&self) -> bool {
        if let Some(evt) = self.schedule.peek() {
            evt.when < SteadyTime::now()
        } else {
            false
        }
    }

    fn fire_event(&mut self) {
        if let Some(evt) = self.schedule.pop() {
            evt.completion_sink.pulse();
        }
    }

    fn ms_until_next_event(&self) -> u32 {
        if let Some(evt) = self.schedule.peek() {
            max(25, min((evt.when - SteadyTime::now()).num_milliseconds() - 4000, 10000))  as u32
        } else {
            1000
        }
    }

    fn run(&mut self) {
        let m = Mutex::new(false);
        let mut g = m.lock().unwrap(); // The mutex isn't poisoned, since we just made it

        loop {
            self.drain_request_queue();

            // Fire off as many events as we are supposed to.
            loop {
                if self.has_event_now() {
                    self.fire_event();
                } else {
                    break;
                }
            }

            let wait_millis = self.ms_until_next_event();

            // unwrap() is safe because the mutex will not be poisoned,
            // since we have not shared it with another thread.
            g = self.trigger.wait_timeout_ms(g, wait_millis).unwrap().0;
        }
    }
}

lazy_static! {
    static ref SCHEDULER_INTERFACE  : Mutex<SchedulingInterface> = {
        let (sender, receiver) = channel();
        let trigger = Arc::new(Condvar::new());
        let trigger2 = trigger.clone();
        thread::spawn(move|| {
            ScheduleWorker::new(trigger2, receiver).run();
        });

        let interface = SchedulingInterface {
            trigger: trigger,
            adder: sender
        };

        Mutex::new(interface)
    };
}

fn add_request(duration_ms: u32) -> Signal {
    let (receiver, sender) = Signal::new();

    let interface = SCHEDULER_INTERFACE.lock().ok().expect("Failed to acquire the global scheduling worker");
    interface.adder.send(SchedulingRequest{
        duration:duration_ms,
        completion_sink:sender,
    }).ok().expect("Failed to send a request to the global scheduling worker");

    interface.trigger.notify_all();

    receiver
}

/// Starts a timer which after `ms` milliseconds will issue a **single** `.send(())` on the other side of the
/// returned `Reciever<()>`.
pub fn oneshot_ms(ms: u32) -> Signal {
    add_request(ms)
}
