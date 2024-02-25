use futures::{Future, FutureExt as _};

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

/// Wait for one of the two futures to complete and return which one completed first.
/// This is a wrapper around the select function from the futures crate for the common
/// case of returning just an output item - dropping both futures at the completion of one.
pub async fn wait_for_one_to_complete<Fut1, Fut2, Out1, Out2>(
    fut1: Fut1,
    fut2: Fut2,
) -> FirstOrSecond<Out1, Out2>
where
    Fut1: Future<Output = Out1> + Unpin,
    Fut2: Future<Output = Out2> + Unpin,
{
    use futures::future::{self, Either};
    match future::select(fut1, fut2).await {
        Either::Left((value_1, _)) => FirstOrSecond::First(value_1),
        Either::Right((value_2, _)) => FirstOrSecond::Second(value_2),
    }
}

pub async fn first_to_complete_or_err<Fut1, Fut2, Fut3, Out1, Out2, Err>(
    fut1: Fut1,
    fut2: Fut2,
    err_fut: Fut3,
) -> Result<FirstOrSecond<Out1, Out2>, Err>
where
    Fut1: Future<Output = Out1>,
    Fut2: Future<Output = Out2>,
    Fut3: Future<Output = Err>,
{
    let fut1 = fut1.fuse();
    let fut2 = fut2.fuse();
    let err_fut = err_fut.fuse();

    futures::pin_mut!(fut1, fut2, err_fut);

    futures::select! {
        result1 = fut1 => Ok(FirstOrSecond::First(result1)),
        result2 = fut2 => Ok(FirstOrSecond::Second(result2)),
        error = err_fut => Err(error),
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
    fn wait_until_voltage_is(&self, value: bool) -> impl Future<Output = ()> + Unpin;
    /// Asynchronously waits for the button to be pressed and returns a ButtonEvent.
    fn wait_until_button_pressed(&mut self) -> impl Future<Output = ButtonEvent> + Unpin;
    /// Asynchronously waits for the button to be released and returns a ButtonEvent.
    fn wait_for_released(&self) -> impl Future<Output = ButtonEvent> + Unpin;
}

/// An empty struct to represent an event that occurred with a timer.
struct TimerEvent;
pub trait AsyncTimer {
    /// Reset the state of the timer to expire again in the future.
    fn reset(&mut self);
    /// Asynchronously waits for the timer to expire and returns a TimerEvent.
    fn wait_expired(&self) -> impl Future<Output = TimerEvent> + Unpin;
}
