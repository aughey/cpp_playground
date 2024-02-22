use crate::{first_to_complete, AsyncIO, AsyncTimer, Light, TimerOrButton};

pub async fn start(mut io: impl AsyncIO, mut timer: impl AsyncTimer) {
    loop {
        io.set_light(Light::Off);
        io.wait_for_pressed().await;
        flash_until_released(&mut io, &mut timer).await;
    }
}

async fn flash_until_released(io: &mut impl AsyncIO, timer: &mut impl AsyncTimer) {
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
}

async fn timer_expired_or_button_released(
    io: &mut impl AsyncIO,
    timer: &impl AsyncTimer,
) -> TimerOrButton {
    first_to_complete(io.wait_for_released(), timer.wait_expired())
        .await
        .into()
}

#[cfg(test)]
mod tests {
    use futures::task::LocalSpawnExt;
    use futures::{executor::LocalPool, Future};
    use std::{cell::RefCell, rc::Rc};

    use crate::{ButtonState, TimerExpired, IO};

    use super::*;

    struct MockAsyncTimer {
        expired: Rc<RefCell<tokio::sync::mpsc::Receiver<bool>>>,
    }
    impl AsyncTimer for MockAsyncTimer {
        fn reset(&mut self) {}
        fn wait_expired(&self) -> impl Future<Output = TimerExpired> + Unpin {
            // just wait for a single expired message
            Box::pin(async {
                self.expired.borrow_mut().recv().await;
                TimerExpired {}
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
        fn wait_for_pressed(&mut self) -> impl Future<Output = ButtonState> + Unpin {
            let rx = self.button_rx.clone();
            Box::pin(async move {
                while let Some(pressed) = rx.borrow_mut().recv().await {
                    self.button = pressed;
                    if pressed {
                        break;
                    }
                }
                ButtonState {}
            })
        }
        fn wait_for_released(&mut self) -> impl Future<Output = ButtonState> + Unpin {
            let rx = self.button_rx.clone();
            Box::pin(async move {
                while let Some(pressed) = rx.borrow_mut().recv().await {
                    self.button = pressed;
                    if !pressed {
                        break;
                    }
                }
                ButtonState {}
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
}
