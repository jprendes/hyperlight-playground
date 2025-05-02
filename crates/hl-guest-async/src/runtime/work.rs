use alloc::collections::{BinaryHeap, VecDeque};
use core::{cmp::Reverse, ops::Deref, time::Duration};

use crate::{
    host::{get_time, poll_read, sleep},
    notify::Notify,
};

struct Unordered<T>(pub T);

impl<T> Ord for Unordered<T> {
    fn cmp(&self, _: &Self) -> core::cmp::Ordering {
        core::cmp::Ordering::Equal
    }
}
impl<T> PartialOrd for Unordered<T> {
    fn partial_cmp(&self, _: &Self) -> Option<core::cmp::Ordering> {
        Some(core::cmp::Ordering::Equal)
    }
}
impl<T> PartialEq for Unordered<T> {
    fn eq(&self, _: &Self) -> bool {
        true
    }
}
impl<T> Eq for Unordered<T> {}

impl<T> Deref for Unordered<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[derive(Default)]
pub(super) struct RuntimeWork {
    timers: BinaryHeap<(Reverse<Duration>, Unordered<Notify>)>,
    ios: VecDeque<Notify>,
}

impl RuntimeWork {
    pub(super) fn work_pending(&self) -> bool {
        !self.timers.is_empty() || !self.ios.is_empty()
    }

    pub(super) fn work(&mut self) {
        let mut timeout = None;
        let mut now = None;
        while let Some((Reverse(deadline), notify)) = self.timers.peek() {
            // we have a scheduled timer
            let now = *now.get_or_insert_with(|| get_time());
            if *deadline <= now {
                // and the timer needed to wake up
                timeout = Some(Duration::ZERO);
                notify.notify_waiters();
                self.timers.pop();
            } else {
                // the timer doesn't need to wake up yet
                // since the times are sorted by deadline,
                // we can stop looking for more timers.
                timeout.get_or_insert_with(|| deadline.saturating_sub(now));
                break;
            }
        }

        if timeout == Some(Duration::ZERO) {
            // we need to wake up immediately as at least one
            // timer is ready to wake up
            return;
        }

        // we would normally have only one io to wake, as we only have
        // one IO channel (stdin). If we have more, then the program has
        // some race condition. This will change if we have more IO
        // channels
        if let Some(notify) = self.ios.front() {
            // we have IO work to do
            // wait for it until a timer timeout (timeout == 0 => no timeout)
            if poll_read(timeout) {
                notify.notify_waiters();
                self.ios.pop_front();
            }
        } else if let Some(timeout) = timeout {
            // no IO work to do, just wait for the timer
            sleep(Some(timeout));
        }
    }

    pub(crate) fn schedule_timer(&mut self, deadline: Duration, notify: Notify) {
        self.timers.push((Reverse(deadline), Unordered(notify)));
    }

    pub(crate) fn schedule_io(&mut self, notify: Notify) {
        self.ios.push_back(notify);
    }
}
