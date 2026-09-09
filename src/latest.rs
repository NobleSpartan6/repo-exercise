//! A one-value mailbox. New measurements replace old ones instead of queuing.
use std::sync::{Arc, Mutex};

pub struct Latest<T> {
    value: Arc<Mutex<Option<T>>>,
}

impl<T> Default for Latest<T> {
    fn default() -> Self {
        Self {
            value: Arc::new(Mutex::new(None)),
        }
    }
}

impl<T> Clone for Latest<T> {
    fn clone(&self) -> Self {
        Self {
            value: Arc::clone(&self.value),
        }
    }
}

impl<T> Latest<T> {
    pub fn publish(&self, value: T) {
        // The lock is never held while taking OS measurements, drawing, or sleeping.
        // Dispose of the replaced value after releasing the lock.
        let previous = self
            .value
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .replace(value);
        drop(previous);
    }

    /// The UI never waits for a worker. A contended sample arrives on a later frame.
    pub fn take(&self) -> Option<T> {
        match self.value.try_lock() {
            Ok(mut value) => value.take(),
            Err(std::sync::TryLockError::Poisoned(error)) => error.into_inner().take(),
            Err(std::sync::TryLockError::WouldBlock) => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_mailbox_is_empty() {
        assert_eq!(Latest::<u8>::default().take(), None);
    }

    #[test]
    fn latest_sample_replaces_older_samples() {
        let mailbox = Latest::default();
        for n in 0..1_000 {
            mailbox.publish(n);
        }
        assert_eq!(mailbox.take(), Some(999));
        assert_eq!(mailbox.take(), None);
    }

    #[test]
    fn cloned_mailbox_transfers_from_a_worker() {
        let mailbox = Latest::default();
        let writer = mailbox.clone();
        std::thread::spawn(move || writer.publish(42))
            .join()
            .unwrap();
        assert_eq!(mailbox.take(), Some(42));
    }

    #[test]
    fn drive_and_cpu_mailboxes_are_independent() {
        let cpu = Latest::default();
        let disk = Latest::<u8>::default();
        let _held = disk.value.lock().unwrap();
        cpu.publish(10);
        assert_eq!(cpu.take(), Some(10));
        assert_eq!(disk.take(), None);
    }

    #[test]
    fn old_samples_are_dropped_not_retained() {
        let mailbox = Latest::default();
        let old = Arc::new(());
        mailbox.publish(Arc::clone(&old));
        assert_eq!(Arc::strong_count(&old), 2);
        mailbox.publish(Arc::new(()));
        assert_eq!(Arc::strong_count(&old), 1);
    }
    #[test]
    fn poisoned_writer_does_not_permanently_silence_readings() {
        let mailbox = Latest::default();
        mailbox.publish(7);
        let writer = mailbox.clone();
        assert!(
            std::thread::spawn(move || {
                let _guard = writer.value.lock().unwrap();
                panic!("simulated worker failure while locked");
            })
            .join()
            .is_err()
        );
        assert_eq!(mailbox.take(), Some(7));
        mailbox.publish(8);
        assert_eq!(mailbox.take(), Some(8));
    }
}
