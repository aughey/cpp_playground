use std::time::Instant;

pub trait Timer {
    fn expired(&self) -> bool;
}
pub trait TimerFactory<T> {
    fn new(&self, timeout: f64) -> T
    where
        T: Timer + Sized;
}

struct SysTimer {
    start: Instant,
    timeout: f64,
}
impl Timer for SysTimer {
    fn expired(&self) -> bool {
        let diff = Instant::now() - self.start;
        diff.as_secs_f64() > self.timeout
    }
}

enum States<T>
where
    T: Timer,
{
    NotPressed,
    BlinkOn(T),
    BlinkOff(T),
    ReleasedButton,
}

pub enum Light {
    On,
    Off,
}
pub trait IO {
    fn button_pressed(&self) -> bool;
    fn button_released(&self) -> bool {
        !self.button_pressed()
    }
    fn set_light(&mut self, state: Light);
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
        while self.handle_state() == true {}
    }
    fn handle_state(&mut self) -> bool {
        match self.state {
            States::NotPressed => {
                if self.io.button_pressed() {
                    self.io.set_light(Light::On);
                    self.state = States::BlinkOn(self.tf.new(1.0));
                    true
                } else {
                    false
                }
            }
            States::BlinkOn(ref timer) => {
                if timer.expired() {
                    self.io.set_light(Light::Off);
                    self.state = States::BlinkOff(self.tf.new(1.0));
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
                    self.state = States::BlinkOn(self.tf.new(1.0));
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
mod tests {
    use super::*;
    use std::rc::Rc;
    use std::rc::Rc;

    struct MockTimer {
        expired: bool,
    }
    impl Timer for MockTimer {
        fn expired(&self) -> bool {
            self.expired
        }
    }
    impl TimerFactory for MockTimer {
        fn new(&self, timeout: f64) -> MockTimer {
            self
        }
    }

    struct MockIO {
        button_pressed: bool,
        light: Light,
    }
    impl IO for MockIO {
        fn button_pressed(&self) -> bool {
            self.button_pressed
        }
        fn set_light(&mut self, state: Light) {
            self.light = state;
        }
    }
}
