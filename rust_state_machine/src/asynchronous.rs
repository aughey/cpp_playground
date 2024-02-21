use std::future::Future;

use crate::{first_to_complete, FirstSecond, Light, IO};

pub trait AsyncIO: IO {
    fn wait_for_pressed(&mut self) -> impl Future<Output = ()> + Unpin;
    fn wait_for_released(&mut self) -> impl Future<Output = ()> + Unpin;
}

pub trait AsyncTimer {
    fn reset(&mut self);
    fn wait_expired(&self) -> impl Future<Output = ()> + Unpin;
}

pub trait AsyncTimerFactory<T> {
    fn new_timer(&self, timeout: f64) -> T
    where
        T: AsyncTimer;
}

pub async fn start<T>(mut io: impl AsyncIO, timer_factory: impl AsyncTimerFactory<T>)
where
    T: AsyncTimer,
{
    loop {
        io.set_light(Light::Off);
        io.wait_for_pressed().await;
        flash_until_released(&mut io, timer_factory.new_timer(1.0)).await;
    }
}

async fn flash_until_released(io: &mut impl AsyncIO, mut timer: impl AsyncTimer) {
    let mut on_off = Light::On;
    loop {
        io.set_light(on_off);
        on_off = match first_to_complete(timer.wait_expired(), io.wait_for_released()).await {
            FirstSecond::First(_) => {
                timer.reset();
                on_off.toggle()
            }
            FirstSecond::Second(_) => break,
        };
    }
}

#[cfg(test)]
mod tests {
    use futures::executor::LocalPool;
    use futures::task::LocalSpawnExt;
    use std::{cell::RefCell, rc::Rc};

    use super::*;

    struct MockAsyncTimerFactory {
        expired: Rc<RefCell<tokio::sync::mpsc::Receiver<bool>>>,
    }
    impl AsyncTimerFactory<MockAsyncTimer> for MockAsyncTimerFactory {
        fn new_timer(&self, _timeout: f64) -> MockAsyncTimer {
            MockAsyncTimer {
                expired: self.expired.clone(),
            }
        }
    }

    struct MockAsyncTimer {
        expired: Rc<RefCell<tokio::sync::mpsc::Receiver<bool>>>,
    }
    impl AsyncTimer for MockAsyncTimer {
        fn reset(&mut self) {}
        fn wait_expired(&self) -> impl Future<Output = ()> + Unpin {
            // just wait for a single expired message
            Box::pin(async {
                self.expired.borrow_mut().recv().await;
                ()
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
        fn wait_for_pressed(&mut self) -> impl Future<Output = ()> + Unpin {
            let rx = self.button_rx.clone();
            Box::pin(async move {
                while let Some(pressed) = rx.borrow_mut().recv().await {
                    self.button = pressed;
                    if pressed {
                        break;
                    }
                }
            })
        }
        fn wait_for_released(&mut self) -> impl Future<Output = ()> + Unpin {
            let rx = self.button_rx.clone();
            Box::pin(async move {
                while let Some(pressed) = rx.borrow_mut().recv().await {
                    self.button = pressed;
                    if !pressed {
                        break;
                    }
                }
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

        let timer_factory = MockAsyncTimerFactory {
            expired: Rc::new(RefCell::new(expire_rx)),
        };

        let mut pool = LocalPool::new();
        pool.spawner()
            .spawn_local(start(io, timer_factory))
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
