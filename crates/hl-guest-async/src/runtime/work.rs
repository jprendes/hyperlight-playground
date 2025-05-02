use alloc::collections::{BinaryHeap, VecDeque};
use core::cmp::Reverse;

use super::controller::WorkController;

use crate::host::{get_time, poll_input, sleep};

#[derive(Default)]
pub(super) struct RuntimeWork {
    pub(super) timers: BinaryHeap<(Reverse<u64>, WorkController)>,
    pub(super) ios: VecDeque<WorkController>,
}

impl RuntimeWork {
    fn peek_timer(&mut self) -> Option<(u64, WorkController)> {
        while let Some((Reverse(deadline), controller)) = self.timers.peek().cloned() {
            match controller.is_cancelled() {
                true => self.pop_timer(),
                false => return Some((deadline, controller)),
            };
        }
        None
    }

    fn pop_timer(&mut self) {
        self.timers.pop();
    }

    fn peek_io(&mut self) -> Option<WorkController> {
        while let Some(controller) = self.ios.front().cloned() {
            match controller.is_cancelled() {
                true => self.pop_io(),
                false => return Some(controller),
            };
        }
        None
    }

    fn pop_io(&mut self) {
        self.ios.pop_front();
    }

    pub(super) fn work_pending(&self) -> bool {
        !self.timers.is_empty() || !self.ios.is_empty()
    }

    pub(super) fn work(&mut self) {
        let mut timeout = None;
        let mut now = None;
        while let Some((deadline, controller)) = self.peek_timer() {
            // we have a scheduled timer
            let now = *now.get_or_insert_with(|| get_time());
            if deadline <= now {
                // and the timer needed to wake up
                timeout = Some(0);
                controller.wake_by_ref();
                self.pop_timer();
            } else {
                // the timer doesn't need to wake up yet
                // since the times are sorted by deadline,
                // we can stop looking for more timers.
                timeout.get_or_insert_with(|| deadline - now);
                break;
            }
        }

        if timeout == Some(0) {
            // we need to wake up immediately as at least one
            // timer is ready to wake up
            return;
        }

        // we would normally have only one io to wake, as we only have
        // one IO channel (stdin). If we have more, then the program has
        // some race condition. This will change if we have more IO
        // channels
        if let Some(controller) = self.peek_io() {
            // we have IO work to do
            // wait for it until a timer timeout (timeout == 0 => no timeout)
            if poll_input(timeout.unwrap_or_default()) {
                controller.wake_by_ref();
                self.pop_io();
            }
        } else if let Some(timeout) = timeout {
            // no IO work to do, just wait for the timer
            sleep(timeout);
        }
    }
}
