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
