use alloc::vec::Vec;
use core::future::Future;
use core::pin::Pin;
use core::task::{Context, Poll};

use super::super::host::try_input;
use crate::runtime::controller::WorkController;

pub async fn read(buf: &mut [u8]) -> usize {
    let data = Read::new(buf.len() as u64).await;
    buf[0..data.len()].copy_from_slice(&data);
    data.len()
}

struct Read {
    count: u64,
    controller: WorkController,
}

impl Read {
    fn new(count: u64) -> Self {
        let controller = WorkController::default();
        crate::runtime::Runtime::global().schedule_io(controller.clone());
        Self { count, controller }
    }
}

impl Future for Read {
    type Output = Vec<u8>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Vec<u8>> {
        let buffer = try_input(self.count);

        if !buffer.is_empty() {
            return Poll::Ready(buffer);
        }

        self.controller.update_waker(cx);

        Poll::Pending
    }
}

impl Drop for Read {
    fn drop(&mut self) {
        self.controller.cancel();
    }
}
