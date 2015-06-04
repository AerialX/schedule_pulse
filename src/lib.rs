//! This crate exposes functionality to create receivers that
//! receive notifications after a specified period of time or at
//! a specified frequency.
//!
//! # Examples
//!
//! At its simplest, oneshot_ms can be used to put the thread to
//! sleep. Unlike with std::thread::sleep, this could be used with
//! Select to be waiting for one of several Receivers to fire.
//!
//! ```
//! # use schedule_pulse::oneshot_ms;
//! # fn sleep_equivalent() {
//! let timer = oneshot_ms(1500);
//! timer.wait().unwrap();
//! println!("1.5 seconds have elapsed.");
//! # }
//! ```


#[macro_use] extern crate lazy_static;
extern crate time;
extern crate pulse;

mod scheduler;

#[cfg(test)] mod test;

pub use scheduler::oneshot_ms;
