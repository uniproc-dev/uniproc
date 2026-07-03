use crate::into_signal::IntoSignal;
use app_core::signal::Signal;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;

pub struct LoopHandle {
    running: Arc<AtomicBool>,
}

impl Drop for LoopHandle {
    fn drop(&mut self) {
        self.running.store(false, Ordering::Relaxed);
    }
}

pub struct Reactor;

impl Reactor {
    pub fn new() -> Self {
        Self
    }

    pub fn add_loop(
        &self,
        interval: impl IntoSignal<u64>,
        active: impl IntoSignal<bool>,
        f: impl FnMut() + 'static,
    ) -> LoopHandle {
        let running = Arc::new(AtomicBool::new(true));
        schedule_next(
            interval.into_signal(),
            active.into_signal(),
            running.clone(),
            f,
        );
        LoopHandle { running }
    }

    pub fn add_heartbeat(
        &self,
        interval: impl IntoSignal<u64>,
        f: impl FnMut() + 'static,
    ) -> LoopHandle {
        self.add_loop(interval, Signal::new(true), f)
    }
}

fn schedule_next(
    interval: Signal<u64>,
    active: Signal<bool>,
    running: Arc<AtomicBool>,
    mut f: impl FnMut() + 'static,
) {
    let delay = Duration::from_millis(interval.get());

    slint::Timer::single_shot(delay, move || {
        if !running.load(Ordering::Relaxed) {
            return;
        }

        if active.get() {
            f();
        }

        schedule_next(interval, active, running, f);
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::sync::Arc;

    fn tick(ms: u64) {
        i_slint_core::tests::slint_mock_elapsed_time(ms);
        slint::platform::update_timers_and_animations();
    }

    #[test]
    fn loop_fires_on_interval() {
        i_slint_backend_testing::init_no_event_loop();
        let mut reactor = Reactor::new();
        let counter = Arc::new(AtomicUsize::new(0));

        let c = counter.clone();
        let _h = reactor.add_loop(Signal::new(100), Signal::new(true), move || {
            c.fetch_add(1, Ordering::SeqCst);
        });

        assert_eq!(counter.load(Ordering::SeqCst), 0);
        tick(101);
        assert_eq!(counter.load(Ordering::SeqCst), 1);
        tick(101);
        assert_eq!(counter.load(Ordering::SeqCst), 2);
    }

    #[test]
    fn loop_stops_on_handle_drop() {
        i_slint_backend_testing::init_no_event_loop();
        let mut reactor = Reactor::new();
        let counter = Arc::new(AtomicUsize::new(0));

        {
            let c = counter.clone();
            let _h = reactor.add_loop(Signal::new(100), Signal::new(true), move || {
                c.fetch_add(1, Ordering::SeqCst);
            });
            tick(101);
            assert_eq!(counter.load(Ordering::SeqCst), 1);
        }

        tick(200);
        assert_eq!(counter.load(Ordering::SeqCst), 1);
    }

    #[test]
    fn loop_respects_active_signal() {
        i_slint_backend_testing::init_no_event_loop();
        let mut reactor = Reactor::new();
        let active = Signal::new(false);
        let counter = Arc::new(AtomicUsize::new(0));

        let c = counter.clone();
        let _h = reactor.add_loop(Signal::new(100), active.clone(), move || {
            c.fetch_add(1, Ordering::SeqCst);
        });

        tick(101);
        assert_eq!(counter.load(Ordering::SeqCst), 0);

        active.set(true);
        tick(101);
        assert_eq!(counter.load(Ordering::SeqCst), 1);

        active.set(false);
        tick(101);
        assert_eq!(counter.load(Ordering::SeqCst), 1);
    }

    #[test]
    fn loop_picks_up_new_interval() {
        i_slint_backend_testing::init_no_event_loop();
        let mut reactor = Reactor::new();
        let interval = Signal::new(1000u64);
        let counter = Arc::new(AtomicUsize::new(0));

        let c = counter.clone();
        let _h = reactor.add_loop(interval.clone(), Signal::new(true), move || {
            c.fetch_add(1, Ordering::SeqCst);
        });

        tick(1001);
        assert_eq!(counter.load(Ordering::SeqCst), 1);

        interval.set(200);

        tick(201);
        assert_eq!(counter.load(Ordering::SeqCst), 2);

        tick(201);
        assert_eq!(counter.load(Ordering::SeqCst), 3);
    }

    #[test]
    fn loop_drop_before_first_tick() {
        i_slint_backend_testing::init_no_event_loop();
        let mut reactor = Reactor::new();
        let counter = Arc::new(AtomicUsize::new(0));

        {
            let c = counter.clone();
            let _h = reactor.add_loop(Signal::new(100), Signal::new(true), move || {
                c.fetch_add(1, Ordering::SeqCst);
            });
        }

        tick(200);
        assert_eq!(counter.load(Ordering::SeqCst), 0);
    }

    // ── add_heartbeat ─────────────────────────────────────────────────────────

    #[test]
    fn heartbeat_always_fires() {
        i_slint_backend_testing::init_no_event_loop();
        let mut reactor = Reactor::new();
        let counter = Arc::new(AtomicUsize::new(0));

        let c = counter.clone();
        let _h = reactor.add_heartbeat(Signal::new(50), move || {
            c.fetch_add(1, Ordering::SeqCst);
        });

        tick(51);
        assert_eq!(counter.load(Ordering::SeqCst), 1);
        tick(51);
        assert_eq!(counter.load(Ordering::SeqCst), 2);
    }

    #[test]
    fn heartbeat_stops_on_drop() {
        i_slint_backend_testing::init_no_event_loop();
        let mut reactor = Reactor::new();
        let counter = Arc::new(AtomicUsize::new(0));

        {
            let c = counter.clone();
            let _h = reactor.add_heartbeat(Signal::new(50), move || {
                c.fetch_add(1, Ordering::SeqCst);
            });
            tick(51);
        }

        tick(200);
        assert_eq!(counter.load(Ordering::SeqCst), 1);
    }
}
