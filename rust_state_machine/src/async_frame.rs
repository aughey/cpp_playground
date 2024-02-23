use std::{
    cell::{Cell, RefCell},
    pin::Pin,
    rc::Rc,
    task::{Context, Poll, Waker},
};

use futures::Future;

pub struct FrameBlocker {
    waker: Rc<RefCell<Option<Waker>>>,
}
impl FrameBlocker {
    pub async fn yield_control(&self) {
        YieldLater::new(self.waker.clone()).await;
    }
}

/// A pool of children that can be woken up in a async/RMS hybrid environment.
#[derive(Default)]
pub struct PollingPool {
    children: Vec<Rc<RefCell<Option<Waker>>>>,
}
impl PollingPool {
    pub fn new_blocker(&mut self) -> FrameBlocker {
        let waker = Rc::new(RefCell::new(None));
        self.children.push(waker.clone());
        FrameBlocker { waker }
    }
    pub fn wake_children(&self) {
        for child in &self.children {
            if let Some(waker) = child.borrow_mut().take() {
                waker.wake_by_ref();
            }
        }
    }
}

struct YieldLater {
    has_yielded: Cell<bool>,
    waker: Rc<RefCell<Option<Waker>>>,
}

impl YieldLater {
    fn new(waker: Rc<RefCell<Option<Waker>>>) -> Self {
        Self {
            waker,
            has_yielded: Cell::new(false),
        }
    }
}

impl Future for YieldLater {
    type Output = ();

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        if self.has_yielded.get() {
            Poll::Ready(())
        } else {
            self.has_yielded.replace(true);
            self.waker.borrow_mut().replace(cx.waker().clone());
            Poll::Pending
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use futures::{executor::LocalPool, task::LocalSpawnExt};
    #[test]
    fn polled_async_behavior() {
        let value = Rc::new(RefCell::new(false));
        let started = Rc::new(RefCell::new(false));
        let exited = Rc::new(RefCell::new(false));
        let loop_count = Rc::new(RefCell::new(0usize));
        let poll_count = Rc::new(RefCell::new(0usize));

        let mut poll_pool = PollingPool::default();
        let blocker = poll_pool.new_blocker();

        let poll_until_true = {
            let value = value.clone();
            let started = started.clone();
            let exited = exited.clone();
            let loop_count = loop_count.clone();
            let poll_count = poll_count.clone();
            async move {
                *started.borrow_mut() = true;
                while !*value.borrow() {
                    let count = poll_count.borrow().to_owned() + 1;
                    *poll_count.borrow_mut() = count;

                    // WAIT!!
                    blocker.yield_control().await;

                    let count = loop_count.borrow().to_owned() + 1;
                    *loop_count.borrow_mut() = count;
                }
                *exited.borrow_mut() = true;
            }
        };

        let mut pool = LocalPool::new();
        pool.spawner()
            .spawn_local(poll_until_true)
            .expect("Failed to spawn poll_until_true");

        let mut frame = move || {
            poll_pool.wake_children();
            pool.run_until_stalled();
        };

        // Pre conditions, async hasn't actually run yet
        assert_eq!(exited.borrow().clone(), false);
        assert_eq!(started.borrow().clone(), false);
        assert_eq!(loop_count.borrow().clone(), 0);

        // Run one frame, it should start and be blocked
        frame();
        assert_eq!(started.borrow().clone(), true);
        assert_eq!(exited.borrow().clone(), false);
        assert_eq!(poll_count.borrow().clone(), 1);
        assert_eq!(loop_count.borrow().clone(), 0);

        // Another frame, counts increment
        frame();
        assert_eq!(started.borrow().clone(), true);
        assert_eq!(exited.borrow().clone(), false);
        assert_eq!(poll_count.borrow().clone(), 2);
        assert_eq!(loop_count.borrow().clone(), 1);

        // Set the value to true, it should exit
        *value.borrow_mut() = true;
        frame();
        assert_eq!(started.borrow().clone(), true);
        assert_eq!(exited.borrow().clone(), true);
        assert_eq!(poll_count.borrow().clone(), 2);
        assert_eq!(loop_count.borrow().clone(), 2);
    }
}
