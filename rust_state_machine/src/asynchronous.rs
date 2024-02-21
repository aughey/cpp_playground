use std::future::Future;

use crate::{first_to_complete, FirstSecond, Light, IO};

pub trait AsyncIO: IO {
    fn wait_for_pressed(&self) -> impl Future<Output = ()> + Unpin;
    fn wait_for_released(&self) -> impl Future<Output = ()> + Unpin;
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
