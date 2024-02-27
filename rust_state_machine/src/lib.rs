use async_frame::FrameBlocker;
use futures::Future;

pub mod async_frame;
pub mod asynchronous;
pub mod sync;

/// Representation of a light state being on or off.
#[derive(PartialEq, Default, Debug, Clone, Copy)]
pub enum Light {
    On,
    #[default]
    Off,
}
impl Light {
    /// Returns the opposite state of the light.
    pub fn toggle(&self) -> Light {
        match self {
            Light::On => Light::Off,
            Light::Off => Light::On,
        }
    }
}
pub trait Timer {
    fn reset(&mut self);
    fn expired(&self) -> bool;
}
pub trait TimerFactory<T> {
    fn new_timer(&self, timeout: f64) -> T
    where
        T: Timer;
}

// Should really separate this out into two traits, one for an abstract button and one for an abstract light.
pub trait IO {
    /// Returns true if the button is currently pressed.
    fn button_pressed(&self) -> bool;
    /// Returns true if the button is currently released.
    fn button_released(&self) -> bool {
        !self.button_pressed()
    }
    /// Set the state of the light.
    fn set_light(&mut self, state: Light);
    fn read_voltage(&self) -> bool;
}

/// Return type of wait_for_one_to_complete indicating which future completed before the other.
pub enum FirstOrSecond<A, B> {
    First(A),
    Second(B),
}

/// A convenience function to convert a FirstOrSecond into a Result.
impl<Good, Bad> Into<Result<Good, Bad>> for FirstOrSecond<Good, Bad> {
    fn into(self) -> Result<Good, Bad> {
        match self {
            FirstOrSecond::First(good) => Ok(good),
            FirstOrSecond::Second(bad) => Err(bad),
        }
    }
}

/// Wait for one of the two futures to complete and return which one completed first.
/// This is a wrapper around the select function from the futures crate for the common
/// case of returning just an output item - dropping both futures at the completion of one.
pub async fn wait_for_one_to_complete<Fut1, Fut2, Out1, Out2>(
    fut1: Fut1,
    fut2: Fut2,
) -> FirstOrSecond<Out1, Out2>
where
    Fut1: Future<Output = Out1>,
    Fut2: Future<Output = Out2>,
{
    use futures::future::{self, Either};
    match future::select(std::pin::pin!(fut1), std::pin::pin!(fut2)).await {
        Either::Left((value_1, _)) => FirstOrSecond::First(value_1),
        Either::Right((value_2, _)) => FirstOrSecond::Second(value_2),
    }
}

/// Given two futures, wait on both and return an Ok result if the good future completes first, or an Err result if the error future completes first.
///
/// This simply converts a FirstOrSecond into an Result where First is Ok and Second is Err.
/// The function exists to better communicate the intent of the operations.
pub async fn wait_for_ok_or_err<Fut, ErrFut, Out, Err>(
    ok_fut: Fut,
    err_fut: ErrFut,
) -> Result<Out, Err>
where
    Fut: Future<Output = Out>,
    ErrFut: Future<Output = Err>,
{
    wait_for_one_to_complete(ok_fut, err_fut).await.into()
}

/// Simulatneously wait all three futures to complete.  Logically the first two futures provide an Ok result, wrapped in a FirstOrSecond enum.  The third future provides an Err result.
///
/// This provides an interface for a common case of needing to monitor two independent asynchronous operations,
/// while having some sort of asynchronous error condition that will fail the operation if it completes first.  
pub async fn first_to_complete_or_err<Fut1, Fut2, Fut3, Out1, Out2, Err>(
    good_fut1: Fut1,
    good_fut2: Fut2,
    err_fut: Fut3,
) -> Result<FirstOrSecond<Out1, Out2>, Err>
where
    Fut1: Future<Output = Out1>,
    Fut2: Future<Output = Out2>,
    Fut3: Future<Output = Err>,
{
    // Simply turn this into a nested pyramid of future selects
    // We combine the two good futures into one, then wait for either
    // the good pair or the error future to complete.
    // If the good pair completes, deconstruct which one completed and return it.
    // If the error future completes, return the error.
    let good_pair = wait_for_one_to_complete(good_fut1, good_fut2);
    match wait_for_ok_or_err(good_pair, err_fut).await {
        Ok(fut1_or_fut2) => match fut1_or_fut2 {
            FirstOrSecond::First(value) => Ok(FirstOrSecond::First(value)),
            FirstOrSecond::Second(value) => Ok(FirstOrSecond::Second(value)),
        },
        Err(err) => Err(err),
    }
}

/// The business logic will wait on either a button press or a timer expiration.  This enum indicates which one completed first.
#[derive(Debug, PartialEq)]
enum TimerOrButton {
    Timer,
    Button,
}

// A slick quality of life to convert a FirstSecond (from wait_for_one_to_complete) into a TimerOrButton
// This is type safe protected because of the return types of the FirstOrSecond enum
impl From<FirstOrSecond<ButtonEvent, TimerEvent>> for TimerOrButton {
    fn from(value: FirstOrSecond<ButtonEvent, TimerEvent>) -> Self {
        match value {
            FirstOrSecond::First(_) => TimerOrButton::Button,
            FirstOrSecond::Second(_) => TimerOrButton::Timer,
        }
    }
}

/// An empty struct to represent an event that occurred with a button.
struct ButtonEvent;
pub trait AsyncIO: IO {
    /// A function that will wait until the voltage on a pin is true or false.
    fn wait_until_voltage_is(&self, value: bool) -> impl Future<Output = ()>;
    /// Asynchronously waits for the button to be pressed and returns a ButtonEvent.
    fn wait_until_button_pressed(&mut self) -> impl Future<Output = ButtonEvent>;
    /// Asynchronously waits for the button to be released and returns a ButtonEvent.
    fn wait_for_released(&self) -> impl Future<Output = ButtonEvent>;
}

/// An empty struct to represent an event that occurred with a timer.
struct TimerEvent;
pub trait AsyncTimer {
    /// Reset the state of the timer to expire again in the future.
    fn reset(&mut self);
    /// Asynchronously waits for the timer to expire and returns a TimerEvent.
    fn wait_expired(&self) -> impl Future<Output = TimerEvent>;
}

struct PollingAsyncTimer<T>
where
    T: Timer,
{
    timer: T,
    blocker: FrameBlocker,
}
impl<T> AsyncTimer for PollingAsyncTimer<T>
where
    T: Timer,
{
    fn reset(&mut self) {
        self.timer.reset();
    }
    fn wait_expired(&self) -> impl Future<Output = TimerEvent> {
        async {
            while !self.timer.expired() {
                self.blocker.yield_control().await;
            }
            TimerEvent {}
        }
    }
}

struct PollingAsyncIO<I>
where
    I: IO,
{
    io: I,
    blocker: FrameBlocker,
}
impl<I> IO for PollingAsyncIO<I>
where
    I: IO,
{
    fn button_pressed(&self) -> bool {
        self.io.button_pressed()
    }
    fn set_light(&mut self, state: Light) {
        self.io.set_light(state);
    }
    fn read_voltage(&self) -> bool {
        self.io.read_voltage()
    }
}
impl<I> AsyncIO for PollingAsyncIO<I>
where
    I: IO,
{
    fn wait_until_button_pressed(&mut self) -> impl Future<Output = ButtonEvent> {
        async {
            while !self.io.button_pressed() {
                self.blocker.yield_control().await;
            }
            ButtonEvent {}
        }
    }
    fn wait_for_released(&self) -> impl Future<Output = ButtonEvent> {
        async {
            while !self.io.button_released() {
                self.blocker.yield_control().await;
            }
            ButtonEvent {}
        }
    }

    fn wait_until_voltage_is(&self, value: bool) -> impl Future<Output = ()> {
        async move {
            while self.io.read_voltage() != value {
                self.blocker.yield_control().await;
            }
        }
    }
}
