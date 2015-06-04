
use scheduler::oneshot_ms;
use time::SteadyTime;

struct ElapsedChecker {
    start: SteadyTime
}

impl ElapsedChecker {
    fn new() -> ElapsedChecker {
        ElapsedChecker {
            start: SteadyTime::now()
        }
    }

    fn check(&self, expected_ms: i64) {
        let actual_elapsed_ms = (SteadyTime::now() - self.start).num_milliseconds();
        assert!(actual_elapsed_ms-100 < expected_ms, "Elapsed too late: {}ms instead of {}ms", actual_elapsed_ms, expected_ms);
        assert!(actual_elapsed_ms+100 > expected_ms, "Elapsed too soon: {}ms instead of {}ms", actual_elapsed_ms, expected_ms);
    }
}

#[test]
fn simple_wait() {
    let checker = ElapsedChecker::new();
    let timer = oneshot_ms(1400);
    timer.wait().unwrap();
    checker.check(1400);
}

#[test]
fn several_concurrent_waits() {
    let checker = ElapsedChecker::new();
    let medium = oneshot_ms(1400);
    let short = oneshot_ms(300);
    let long = oneshot_ms(2000);
    short.wait().unwrap();
    checker.check(300);

    medium.wait().unwrap();
    checker.check(1400);

    long.wait().unwrap();
    checker.check(2000);
}

#[test]
fn several_concurrent_waits_misordered() {
    let checker = ElapsedChecker::new();
    let medium = oneshot_ms(1400);
    let short = oneshot_ms(300);
    let long = oneshot_ms(2000);

    // We wait for the last timer before we check the others,
    // so they should all recv immediately after the first one.
    long.wait().unwrap();
    checker.check(2000);

    short.wait().unwrap();
    checker.check(2000);

    medium.wait().unwrap();
    checker.check(2000);
}
