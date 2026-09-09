//! Timed, opt-in power assertion. A sleeping worker owns and releases the OS guard.
use crate::latest::Latest;
use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
    mpsc,
};
use std::time::{Duration, Instant};
pub struct Awake {
    stop: Option<mpsc::Sender<()>>,
    active: Arc<AtomicBool>,
    finished: Arc<AtomicBool>,
    pub error: Latest<String>,
    pub until: Instant,
}
impl Awake {
    pub fn start(duration: Duration, wake: impl Fn() + Send + 'static) -> Result<Self, String> {
        Self::spawn(duration, wake, || {
            keepawake::Builder::default()
                .display(true)
                .idle(true)
                .sleep(false)
                .reason("Burrow: user-requested timed screen-on session")
                .app_name("Burrow")
                .app_reverse_domain("io.github.noblespartan6.burrow")
                .create()
                .map_err(|e| e.to_string())
        })
    }
    fn spawn<G: 'static>(
        duration: Duration,
        wake: impl Fn() + Send + 'static,
        create: impl FnOnce() -> Result<G, String> + Send + 'static,
    ) -> Result<Self, String> {
        if duration.is_zero() || duration > Duration::from_secs(3600) {
            return Err("Choose a screen-on session of 1–60 minutes.".into());
        }
        let (tx, rx) = mpsc::channel();
        let active = Arc::new(AtomicBool::new(false));
        let finished = Arc::new(AtomicBool::new(false));
        let error = Latest::default();
        let (a, f, e) = (active.clone(), finished.clone(), error.clone());
        std::thread::Builder::new()
            .name("burrow-screen-on".into())
            .spawn(move || {
                match std::panic::catch_unwind(std::panic::AssertUnwindSafe(create))
                    .unwrap_or_else(|_| Err("The power provider stopped unexpectedly.".into()))
                {
                    Ok(guard) => {
                        a.store(true, Ordering::Release);
                        wake();
                        let _ = rx.recv_timeout(duration);
                        drop(guard);
                    }
                    Err(message) => {
                        e.publish(message);
                    }
                }
                a.store(false, Ordering::Release);
                f.store(true, Ordering::Release);
                wake();
            })
            .map_err(|e| e.to_string())?;
        Ok(Self {
            stop: Some(tx),
            active,
            finished,
            error,
            until: Instant::now() + duration,
        })
    }
    pub fn active(&self) -> bool {
        self.active.load(Ordering::Acquire)
    }
    pub fn finished(&self) -> bool {
        self.finished.load(Ordering::Acquire)
    }
}
impl Drop for Awake {
    fn drop(&mut self) {
        if let Some(tx) = self.stop.take() {
            let _ = tx.send(());
        }
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    struct Guard(Arc<AtomicBool>);
    impl Drop for Guard {
        fn drop(&mut self) {
            self.0.store(true, Ordering::Relaxed);
        }
    }
    #[test]
    fn timer_releases_the_guard() {
        let dropped = Arc::new(AtomicBool::new(false));
        let d = dropped.clone();
        let session = Awake::spawn(Duration::from_millis(20), || {}, move || Ok(Guard(d))).unwrap();
        for _ in 0..100 {
            if session.finished() {
                break;
            }
            std::thread::sleep(Duration::from_millis(5));
        }
        assert!(session.finished());
        assert!(dropped.load(Ordering::Relaxed));
        assert!(!session.active());
    }
    #[test]
    fn stop_releases_the_guard() {
        let dropped = Arc::new(AtomicBool::new(false));
        let d = dropped.clone();
        let session = Awake::spawn(Duration::from_secs(60), || {}, move || Ok(Guard(d))).unwrap();
        drop(session);
        for _ in 0..100 {
            if dropped.load(Ordering::Relaxed) {
                break;
            }
            std::thread::sleep(Duration::from_millis(5));
        }
        assert!(dropped.load(Ordering::Relaxed));
    }
    #[test]
    fn failed_provider_is_not_marked_active() {
        let session = Awake::spawn(
            Duration::from_secs(1),
            || {},
            || Err::<(), _>("not supported".into()),
        )
        .unwrap();
        for _ in 0..100 {
            if session.finished() {
                break;
            }
            std::thread::sleep(Duration::from_millis(5));
        }
        assert!(session.finished());
        assert!(!session.active());
        assert_eq!(session.error.take().unwrap(), "not supported");
    }
}
