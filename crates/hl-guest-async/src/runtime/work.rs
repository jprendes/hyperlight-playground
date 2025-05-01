use core::cmp::Reverse;
use alloc::collections::{BinaryHeap, VecDeque};

use super::controller::WorkController;

use crate::host::{get_time, poll_input, sleep};

#[derive(Default)]
pub(super) struct RuntimeWork {
    pub(super) timers: BinaryHeap<(Reverse<u64>, WorkController)>,
    pub(super) ios: VecDeque<WorkController>,
}

impl RuntimeWork {
    fn prune(&mut self) {
        // Remove all cancelled work from the front of queues
        while self
            .timers
            .peek()
            .is_some_and(|(_, c)| c.is_cancelled())
        {
            self.timers.pop();
        }
        while self
            .ios
            .front()
            .is_some_and(|c| c.is_cancelled())
        {
            self.ios.pop_front();
        }
    }

    pub(super) fn work(&mut self) -> bool {
        self.prune();

        if self.timers.is_empty() && self.ios.is_empty() {
            return false;
        }

        let mut timeout: u64 = 0;
        if let Some((Reverse(deadline), controller)) = self.timers.peek() {
            let now = get_time();
            if *deadline <= now {
                controller.wake_by_ref();
                self.timers.pop();
                return true;
            } else {
                timeout = *deadline - now;
            }
        }

        if let Some(controller) = self.ios.front() {
            if poll_input(timeout) {
                controller.wake_by_ref();
                self.ios.pop_front();
            }
        } else {
            sleep(timeout);
        }

        true
    }
}
