use futures::Future;

pub mod asynchronous;
pub mod sync;

#[derive(PartialEq, Debug, Clone, Copy)]
pub enum Light {
    On,
    Off,
}
impl Light {
    pub fn toggle(&self) -> Light {
        match self {
            Light::On => Light::Off,
            Light::Off => Light::On,
        }
    }
}

pub trait IO {
    fn button_pressed(&self) -> bool;
    fn button_released(&self) -> bool {
        !self.button_pressed()
    }
    fn set_light(&mut self, state: Light);
}

enum FirstSecond<A, B> {
    First(A),
    Second(B),
}
async fn first_to_complete<Fut1, Fut2, Out1, Out2>(
    fut1: Fut1,
    fut2: Fut2,
) -> FirstSecond<Out1, Out2>
where
    Fut1: Future<Output = Out1> + Unpin,
    Fut2: Future<Output = Out2> + Unpin,
{
    use futures::future::{self, Either};
    match future::select(fut1, fut2).await {
        Either::Left((value_1, _)) => FirstSecond::First(value_1),
        Either::Right((value_2, _)) => FirstSecond::Second(value_2),
    }
}

#[derive(Debug, PartialEq)]
enum TimerOrButton {
    Timer,
    Button,
}

// A slick quality of life to convert a FirstSecond into a TimerOrButton
// This is type safe protected because of the return types of the FirstOrSecond enum
impl From<FirstSecond<ButtonState, TimerExpired>> for TimerOrButton {
    fn from(value: FirstSecond<ButtonState, TimerExpired>) -> Self {
        match value {
            FirstSecond::First(_) => TimerOrButton::Button,
            FirstSecond::Second(_) => TimerOrButton::Timer,
        }
    }
}

struct ButtonState;
pub trait AsyncIO: IO {
    fn wait_for_pressed(&mut self) -> impl Future<Output = ButtonState> + Unpin;
    fn wait_for_released(&mut self) -> impl Future<Output = ButtonState> + Unpin;
}

struct TimerExpired;
pub trait AsyncTimer {
    fn reset(&mut self);
    fn wait_expired(&self) -> impl Future<Output = TimerExpired> + Unpin;
}
