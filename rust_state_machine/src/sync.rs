use std::time::Instant;

use crate::{Light, IO};

pub trait Timer {
    fn reset(&mut self);
    fn expired(&self) -> bool;
}
pub trait TimerFactory<T> {
    fn new_timer(&self, timeout: f64) -> T
    where
        T: Timer;
}

struct SysTimer {
    start: Instant,
    timeout: f64,
}
impl Timer for SysTimer {
    fn reset(&mut self) {
        self.start = Instant::now();
    }
    fn expired(&self) -> bool {
        let diff = Instant::now() - self.start;
        diff.as_secs_f64() > self.timeout
    }
}

#[derive(Debug)]
pub(crate) enum States<T>
where
    T: Timer,
{
    NotPressed,
    BlinkOn(T),
    BlinkOff(T),
    ReleasedButton,
}

pub struct StateMachineSync<TF, T, I>
where
    TF: TimerFactory<T>,
    T: Timer,
    I: IO,
{
    state: States<T>,
    io: I,
    tf: TF,
}
impl<TF, T, I> StateMachineSync<TF, T, I>
where
    TF: TimerFactory<T>,
    T: Timer,
    I: IO,
{
    pub fn new(io: I, tf: TF) -> Self {
        Self {
            state: States::NotPressed,
            io,
            tf,
        }
    }
    pub fn do_work(&mut self) {
        while self.handle_state() {}
    }
    #[cfg(test)]
    pub(crate) fn current_state(&self) -> &States<T> {
        &self.state
    }
    fn handle_state(&mut self) -> bool {
        match self.state {
            States::NotPressed => {
                if self.io.button_pressed() {
                    self.io.set_light(Light::On);
                    self.state = States::BlinkOn(self.tf.new_timer(1.0));
                    true
                } else {
                    false
                }
            }
            States::BlinkOn(ref timer) => {
                if timer.expired() {
                    self.io.set_light(Light::Off);
                    self.state = States::BlinkOff(self.tf.new_timer(1.0));
                    true
                } else if self.io.button_released() {
                    self.state = States::ReleasedButton;
                    true
                } else {
                    false
                }
            }
            States::BlinkOff(ref timer) => {
                if timer.expired() {
                    self.io.set_light(Light::On);
                    self.state = States::BlinkOn(self.tf.new_timer(1.0));
                    true
                } else if self.io.button_released() {
                    self.state = States::ReleasedButton;
                    true
                } else {
                    false
                }
            }
            States::ReleasedButton => {
                self.io.set_light(Light::Off);
                self.state = States::NotPressed;
                true
            }
        }
    }
}

#[cfg(test)]
pub mod tests {
    use crate::Light;

    use super::*;
    use std::cell::RefCell;
    use std::rc::Rc;

    #[derive(Clone, Default, Debug)]
    pub struct MockTimer {
        expired: Rc<RefCell<bool>>,
    }
    impl MockTimer {
        pub fn new(expired: Rc<RefCell<bool>>) -> Self {
            Self { expired }
        }
    }
    impl Timer for MockTimer {
        fn expired(&self) -> bool {
            *self.expired.borrow()
        }

        fn reset(&mut self) {
            self.expired.replace(false);
        }
    }
    impl TimerFactory<MockTimer> for MockTimer {
        fn new_timer(&self, _timeout: f64) -> MockTimer {
            *self.expired.borrow_mut() = false;
            self.clone()
        }
    }

    #[derive(Default)]
    pub struct MockIO {
        button_pressed: Rc<RefCell<bool>>,
        voltage: Rc<RefCell<bool>>,
        light: Rc<RefCell<Light>>,
    }
    impl MockIO {
        pub fn new(
            button_pressed: Rc<RefCell<bool>>,
            light: Rc<RefCell<Light>>,
            voltage: Rc<RefCell<bool>>,
        ) -> Self {
            Self {
                button_pressed,
                light,
                voltage,
            }
        }
    }
    impl IO for MockIO {
        fn button_pressed(&self) -> bool {
            *self.button_pressed.borrow()
        }
        fn set_light(&mut self, state: Light) {
            *self.light.borrow_mut() = state;
        }
        fn read_voltage(&self) -> bool {
            *self.voltage.borrow()
        }
    }

    #[test]
    fn test_state_machine() {
        let button_pressed = Rc::new(RefCell::new(false));
        let light = Rc::new(RefCell::new(Light::Off));
        let voltage = Rc::new(RefCell::new(true));
        let io = MockIO {
            button_pressed: button_pressed.clone(),
            light: light.clone(),
            voltage: voltage.clone(),
        };
        let expired = Rc::new(RefCell::new(false));

        let mut behavior = StateMachineSync::new(
            io,
            MockTimer {
                expired: expired.clone(),
            },
        );

        for _ in 0..100 {
            behavior.do_work();
            assert_eq!(*light.borrow_mut(), Light::Off);
            assert!(
                matches!(*behavior.current_state(), States::NotPressed),
                "Found {:?}",
                behavior.current_state()
            );
        }

        button_pressed.replace(true);
        for _ in 0..100 {
            behavior.do_work();
            assert_eq!(*light.borrow(), Light::On);
            assert!(
                matches!(*behavior.current_state(), States::BlinkOn(_)),
                "Found {:?}",
                behavior.current_state()
            );
        }

        *expired.borrow_mut() = true;

        for _ in 0..100 {
            behavior.do_work();
            assert_eq!(*light.borrow(), Light::Off);
            assert!(
                matches!(*behavior.current_state(), States::BlinkOff(_)),
                "Found {:?}",
                behavior.current_state()
            );
        }

        button_pressed.replace(false);

        // Should go to released button and immediate to not pressed (without seeing released button)

        for _ in 0..100 {
            behavior.do_work();
            assert_eq!(*light.borrow(), Light::Off);
            assert!(
                matches!(*behavior.current_state(), States::NotPressed),
                "Found {:?}",
                behavior.current_state()
            );
        }
    }
}
