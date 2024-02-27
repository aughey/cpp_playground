use crate::{
    first_to_complete_or_err, wait_for_one_to_complete, AsyncIO, AsyncTimer, FirstOrSecond, Light,
    TimerOrButton,
};

/// The entry point for the flashing behavior of a light when a button is pressed.
/// This is the top level of the state machine providing the sequence of events to
/// act one according to the defined business logic.
///
/// Business logic says to wait for the button to be pressed, then flash the light
/// until the button is released.
pub async fn start(
    mut io: impl AsyncIO,
    mut timer: impl AsyncTimer,
    mut transition_timer: impl AsyncTimer,
) -> Result<(), &'static str> {
    // initial light state is off.
    io.set_light(Light::Off);

    loop {
        io.wait_until_button_pressed().await;
        flash_until_button_released(&mut io, &mut timer, &mut transition_timer).await?;
    }
}

// This async function will internally run a state machine that will
// wait for up to transition_timer duration until the reading on the
// light pin transitions to the expected_reading.  If the transition
// happens before the timer expires, the function will continue to
// monitor that the reading stays in that state for the duration of
// the async lifetime.
//
// If the timer expires before the transition the function will return
// indicating an error.  If the voltage transitions away from the expected
// reading after the transition, the function will return indicating an error.
//
// The "good condition" is that this function never returns.  Returning from
// this function indicates that the above conditions failed in some way.
pub async fn monitor_voltage_transition(
    io: &impl AsyncIO,
    transition_timer: &impl AsyncTimer,
    expected_reading: bool,
) -> &'static str {
    // Wait until the reading goes to the expected value or the timer expires
    if let FirstOrSecond::Second(_) = wait_for_one_to_complete(
        io.wait_until_voltage_is(expected_reading),
        transition_timer.wait_expired(),
    )
    .await
    {
        return "Timer expired before voltage transition";
    }

    // It transitioned to the expected reading, now wait until it transitions
    // back down
    _ = io.wait_until_voltage_is(!expected_reading).await;
    "Voltage transitioned away from expected reading after transition."
}

/// Internal state logic for flashing the light until the button is released.
/// Internal to this function will keep track of the current light state and
/// toggle the light state every time the timer expires.  If the button is released
/// at any time, this flashing behavior will stop.
async fn flash_until_button_released(
    io: &mut impl AsyncIO,
    timer: &mut impl AsyncTimer,
    transition_timer: &mut impl AsyncTimer,
) -> Result<(), &'static str> {
    // Setup our initial state of the light being on and the timer being reset
    // Keep track of whether the light is on or off
    let mut light_state = Light::On;
    // Turn the light on
    io.set_light(light_state);
    // Reset the timer so we get a full blink
    timer.reset();
    transition_timer.reset();

    // Loop until the timer expires or the button is released.
    // Keep looping if the thing that happened was the timer expiring.
    while TimerOrButton::Timer
        == first_to_complete_or_err(
            io.wait_for_released(), // Good, if the button is released, we're done
            timer.wait_expired(),   // Good, if the timer expires, we need to flip the light
            monitor_voltage_transition(io, transition_timer, true), // Bad, if the voltage transitions away from the expected reading
        )
        .await?
        .into()
    {
        // Inside the loop the timer expired, reset timer, flip light state, and set light
        timer.reset();
        transition_timer.reset();
        light_state = light_state.toggle();
        io.set_light(light_state);
    }

    // When the button is released, set the light back to off.
    io.set_light(Light::Off);
    Ok(())
}

#[allow(dead_code)]
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
    use futures::executor::LocalPool;
    use futures::task::LocalSpawnExt;
    use std::cell::Cell;
    use std::{cell::RefCell, rc::Rc};

    use crate::sync::tests::{MockIO, MockTimer};
    use crate::{PollingAsyncIO, PollingAsyncTimer};

    use super::*;

    #[test]
    fn test_flash_behavior() {
        // A bit of setup to rig up the mock IO and timer to work in this async environment
        let light = Rc::new(RefCell::new(Light::Off));

        let mut pool_poll = crate::async_frame::PollingPool::default();
        let button_pressed = Rc::new(RefCell::new(false));
        let voltage = Rc::new(RefCell::new(true));
        let io = PollingAsyncIO {
            io: MockIO::new(button_pressed.clone(), light.clone(), voltage),
            blocker: pool_poll.new_blocker(),
        };

        let time_expired = Rc::new(RefCell::new(false));
        let transition_time_expired = Rc::new(RefCell::new(false));
        let timer = PollingAsyncTimer {
            timer: MockTimer::new(time_expired.clone()),
            blocker: pool_poll.new_blocker(),
        };
        let transition_timer = PollingAsyncTimer {
            timer: MockTimer::new(transition_time_expired.clone()),
            blocker: pool_poll.new_blocker(),
        };

        let mut pool = LocalPool::new();
        let run_error = Rc::new(Cell::new(None));
        {
            let run_error = run_error.clone();
            pool.spawner()
                .spawn_local(async move {
                    if let Err(e) = start(io, timer, transition_timer).await {
                        run_error.replace(Some(e));
                    }
                    ()
                })
                .expect("Failed to spawn start");
        }

        let mut frame = move || {
            pool_poll.wake_children();
            pool.run_until_stalled();
        };

        // Should be off
        for _ in 0..10 {
            frame();
            assert_eq!(*light.borrow(), Light::Off);
        }

        // simulate button press
        button_pressed.replace(true);

        assert_eq!(*light.borrow(), Light::Off);
        for i in 0..10 {
            frame();
            assert_eq!(*light.borrow(), Light::On, "Failed on iteration {}", i);
        }

        // Simulate a timer expiration
        *time_expired.borrow_mut() = true;

        // Should switch to off
        for _ in 0..10 {
            frame();
            assert_eq!(*light.borrow(), Light::Off);
        }
        assert_eq!(time_expired.borrow().clone(), false);

        // And back on again
        *time_expired.borrow_mut() = true;
        for _ in 0..10 {
            frame();
            assert_eq!(*light.borrow(), Light::On);
        }

        // And release the button, should go off for good
        button_pressed.replace(false);
        for _ in 0..10 {
            frame();
            assert_eq!(*light.borrow(), Light::Off);
        }
    }

    #[test]
    fn test_voltage_monitor() {
        let timer_expired = Rc::new(RefCell::new(false));
        let voltage_value = Rc::new(RefCell::new(true));
        let voltage_errored = Rc::new(Cell::new(None));

        let reset = || {
            timer_expired.replace(false);
            voltage_value.replace(true);
            voltage_errored.replace(None);

            let mut pool_poll = crate::async_frame::PollingPool::default();
            let timer = PollingAsyncTimer {
                timer: MockTimer::new(timer_expired.clone()),
                blocker: pool_poll.new_blocker(),
            };
            let io = PollingAsyncIO {
                io: MockIO::new(
                    Rc::new(RefCell::new(false)),
                    Rc::new(RefCell::new(Light::Off)),
                    voltage_value.clone(),
                ),
                blocker: pool_poll.new_blocker(),
            };

            let mut pool = LocalPool::new();
            let voltage_errored = voltage_errored.clone();
            pool.spawner()
                .spawn_local(async move {
                    let e = monitor_voltage_transition(&io, &timer, true).await;
                    voltage_errored.replace(Some(e));
                    ()
                })
                .expect("must spawn");

            move || {
                pool_poll.wake_children();
                pool.run_until_stalled();
            }
        };
        let mut frame = reset();

        // Start low, should wait for the transition
        voltage_value.replace(false);
        frame();
        assert_eq!(voltage_errored.get(), None);

        // expire timer and should fail.
        *timer_expired.borrow_mut() = true;
        frame();
        assert_eq!(
            voltage_errored.get(),
            Some("Timer expired before voltage transition")
        );

        let mut frame = reset();
        frame();
        assert_eq!(voltage_errored.get(), None);

        // Change our voltage, and internally should transition to the waiting
        // for the voltage to transition back
        frame();
        assert_eq!(voltage_errored.get(), None);

        // Now drop voltage and see that it fails
        voltage_value.replace(false);
        frame();
        assert_eq!(
            voltage_errored.get(),
            Some("Voltage transitioned away from expected reading after transition.")
        );

        // Just for fun, if the timer expires after transition, aok
        let mut frame = reset();
        voltage_value.replace(true);
        frame();
        assert_eq!(voltage_errored.get(), None);

        // Should have transitioned to the waiting for the voltage to transition back
        *timer_expired.borrow_mut() = true;
        frame();
        assert_eq!(voltage_errored.get(), None);
    }
}
