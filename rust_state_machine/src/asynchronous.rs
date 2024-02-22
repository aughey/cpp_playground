use crate::{wait_for_one_to_complete, AsyncIO, AsyncTimer, Light, TimerOrButton};

/// The entry point for the flashing behavior of a light when a button is pressed.
/// This is the top level of the state machine providing the sequence of events to
/// act one according to the defined business logic.
///
/// Business logic says to wait for the button to be pressed, then flash the light
/// until the button is released.
pub async fn start(mut io: impl AsyncIO, mut timer: impl AsyncTimer) {
    // initial light state is off.
    io.set_light(Light::Off);

    loop {
        io.wait_until_button_pressed().await;
        flash_until_button_released(&mut io, &mut timer).await;
    }
}

/// Internal state logic for flashing the light until the button is released.
/// Internal to this function will keep track of the current light state and
/// toggle the light state every time the timer expires.  If the button is released
/// at any time, this flashing behavior will stop.
async fn flash_until_button_released(io: &mut impl AsyncIO, timer: &mut impl AsyncTimer) {
    // Setup our initial state of the light being on and the timer being reset
    // Keep track of whether the light is on or off
    let mut light_state = Light::On;
    // Turn the light on
    io.set_light(light_state);
    // Reset the timer so we get a full blink
    timer.reset();

    // Loop until the timer expires or the button is released.
    // Keep looping if the thing that happened was the timer expiring.
    while TimerOrButton::Timer == timer_expired_or_button_released(io, timer).await {
        // Inside the loop the timer expired, reset timer, flip light state, and set light
        timer.reset();
        light_state = light_state.toggle();
        io.set_light(light_state);
    }

    // When the button is released, set the light back to off.
    io.set_light(Light::Off);
}

async fn timer_expired_or_button_released(
    io: &mut impl AsyncIO,
    timer: &impl AsyncTimer,
) -> TimerOrButton {
    wait_for_one_to_complete(io.wait_for_released(), timer.wait_expired())
        .await
        .into()
}

#[cfg(test)]
mod tests {
    use futures::task::LocalSpawnExt;
    use futures::{executor::LocalPool, Future};
    use std::cell::Cell;
    use std::pin::Pin;
    use std::task::{Context, Poll};
    use std::{cell::RefCell, rc::Rc};

    use crate::{ButtonEvent, TimerEvent, IO};

    use super::*;

    struct MockAsyncTimer {
        expired: Rc<RefCell<tokio::sync::mpsc::Receiver<bool>>>,
    }
    impl AsyncTimer for MockAsyncTimer {
        fn reset(&mut self) {}
        fn wait_expired(&self) -> impl Future<Output = TimerEvent> + Unpin {
            // just wait for a single expired message
            Box::pin(async {
                self.expired.borrow_mut().recv().await;
                TimerEvent {}
            })
        }
    }

    struct MockAsyncIO {
        button_rx: Rc<RefCell<tokio::sync::mpsc::Receiver<bool>>>,
        button: bool,
        light: Rc<RefCell<Light>>,
    }
    impl IO for MockAsyncIO {
        fn button_pressed(&self) -> bool {
            self.button
        }
        fn set_light(&mut self, state: Light) {
            *self.light.borrow_mut() = state;
        }
    }
    impl AsyncIO for MockAsyncIO {
        fn wait_until_button_pressed(&mut self) -> impl Future<Output = ButtonEvent> + Unpin {
            let rx = self.button_rx.clone();
            Box::pin(async move {
                while let Some(pressed) = rx.borrow_mut().recv().await {
                    self.button = pressed;
                    if pressed {
                        break;
                    }
                }
                ButtonEvent {}
            })
        }
        fn wait_for_released(&mut self) -> impl Future<Output = ButtonEvent> + Unpin {
            let rx = self.button_rx.clone();
            Box::pin(async move {
                while let Some(pressed) = rx.borrow_mut().recv().await {
                    self.button = pressed;
                    if !pressed {
                        break;
                    }
                }
                ButtonEvent {}
            })
        }
    }

    #[test]
    fn test_flash_behavior() {
        // A bit of setup to rig up the mock IO and timer to work in this async environment
        let light = Rc::new(RefCell::new(Light::Off));

        let (button_tx, button_rx) = tokio::sync::mpsc::channel(1);

        let button_rx = Rc::new(RefCell::new(button_rx));

        let io = MockAsyncIO {
            button_rx,
            button: false,
            light: light.clone(),
        };

        let (expire_tx, expire_rx) = tokio::sync::mpsc::channel(1);

        let timer = MockAsyncTimer {
            expired: Rc::new(RefCell::new(expire_rx)),
        };

        let mut pool = LocalPool::new();
        pool.spawner()
            .spawn_local(start(io, timer))
            .expect("Failed to spawn start");

        pool.try_run_one();

        assert_eq!(*light.borrow(), Light::Off);

        // simulate button press
        button_tx.blocking_send(true).unwrap();

        assert_eq!(*light.borrow(), Light::Off);
        for _ in 0..10 {
            pool.try_run_one();
            assert_eq!(*light.borrow(), Light::On);
        }

        // Simulate a timer expiration
        expire_tx.blocking_send(true).unwrap();

        // Should switch to off
        for _ in 0..10 {
            pool.try_run_one();
            assert_eq!(*light.borrow(), Light::Off);
        }

        // And back on again
        expire_tx.blocking_send(true).unwrap();
        for _ in 0..10 {
            pool.try_run_one();
            assert_eq!(*light.borrow(), Light::On);
        }

        // And release the button, should go off for good
        button_tx.blocking_send(false).unwrap();
        for _ in 0..10 {
            pool.try_run_one();
            assert_eq!(*light.borrow(), Light::Off);
        }
    }

    struct YieldNow {
        has_yielded: Cell<bool>,
    }

    impl YieldNow {
        fn new() -> Self {
            Self {
                has_yielded: Cell::new(false),
            }
        }
    }

    impl Future for YieldNow {
        type Output = ();

        fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
            if self.has_yielded.get() {
                Poll::Ready(())
            } else {
                self.has_yielded.set(true);
                cx.waker().wake_by_ref();
                Poll::Pending
            }
        }
    }

    fn yield_now() -> YieldNow {
        YieldNow::new()
    }

    #[test]
    fn polled_async_behavior() {
        let value = Rc::new(RefCell::new(false));
        let started = Rc::new(RefCell::new(false));
        let exited = Rc::new(RefCell::new(false));
        let loop_count = Rc::new(RefCell::new(0usize));

        let poll_until_true = {
            let value = value.clone();
            let started = started.clone();
            let exited = exited.clone();
            let loop_count = loop_count.clone();
            async move {
                *started.borrow_mut() = true;
                while !*value.borrow() {
                    //futures_timer::Delay::new(std::time::Duration::from_millis(0)).await;
                    yield_now().await;

                    let count = loop_count.borrow().clone() + 1;
                    *loop_count.borrow_mut() = count;
                }
                *exited.borrow_mut() = true;
            }
        };

        let mut pool = LocalPool::new();
        pool.spawner()
            .spawn_local(poll_until_true)
            .expect("Failed to spawn poll_until_true");

        assert_eq!(exited.borrow().clone(), false);
        assert_eq!(started.borrow().clone(), false);
        assert_eq!(loop_count.borrow().clone(), 0);
        // pool.run_until_stalled();
        // assert_eq!(started.borrow().clone(), true);
        // assert_eq!(exited.borrow().clone(), false);
        // assert_eq!(loop_count.borrow().clone(), 1);
        // *value.borrow_mut() = true;
        // assert_eq!(exited.borrow().clone(), false);
        // pool.run_until_stalled();
        // assert_eq!(exited.borrow().clone(), true);
    }
}
