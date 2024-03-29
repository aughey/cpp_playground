#pragma GCC diagnostic ignored "-Wunused-parameter"

#include <gtest/gtest.h>
#include <stdint.h>

enum class FlashResult
{
    Released,
    Timer
};

enum class OnOff
{
    On,
    Off
};
static OnOff toggle(OnOff value)
{
    return (value == OnOff::On) ? OnOff::Off : OnOff::On;
}

class IIO
{
public:
    virtual void set_light(OnOff on_or_off) = 0;
    virtual bool button_pressed() = 0;
    virtual bool button_released() = 0;
};

class TestIO : public IIO
{
public:
    void set_light(OnOff on_or_off) override
    {
        light_value = on_or_off;
    }
    bool button_pressed() override
    {
        // Return true if the button is pressed
        return button_pressed_value;
    }
    bool button_released() override
    {
        // Return true if the button is released
        return !button_pressed_value;
    }
    OnOff light_value = OnOff::Off;
    bool button_pressed_value = false;
};

class ITimer
{
public:
    virtual ITimer &reset(double seconds) = 0;
    virtual bool expired() const = 0;
};

class TestTimer : public ITimer
{
public:
    TestTimer() {}
    TestTimer(double seconds) {}
    ITimer &reset(double seconds) override
    {
        expired_value = false;
        return *this;
    }
    bool expired() const override
    {
        // Return true if the timer has expired
        return expired_value;
    }
    bool expired_value = false;
};

/// Busy-waits on the two options.  Returns RELEASED if button released, ITimer if ITimer expired
FlashResult button_released_or_timer_expired(IIO &io, ITimer &timer)
{
    while (true)
    {
        if (io.button_released())
        {
            return FlashResult::Released;
        }
        if (timer.expired())
        {
            return FlashResult::Timer;
        }
    }
}

void flash_until_button_released(IIO &io, ITimer &timer)
{
    // Setup our initial state of the light being on and the timer being reset
    // Keep track of whether the light is on or off
    OnOff light_state = OnOff::On;
    // Turn the light on
    io.set_light(light_state);
    // Reset the timer so we get a full blink
    timer.reset(1.0);

    // Loop until the button is released or the timer expires
    // Keep looping if the thing that happened was the timer expiring.
    while (FlashResult::Timer == button_released_or_timer_expired(io, timer))
    {
        // Inside the loop the timer expired. Reset timer, flip the light state, and set the light
        timer.reset(1.0);
        light_state = toggle(light_state);
        io.set_light(light_state);
    }

    // Before we exit, turn the light back off.
    io.set_light(OnOff::Off);
}

void wait_until_button_pressed(IIO &io)
{
    while (true)
    {
        if (io.button_pressed())
        {
            break;
        }
    }
}

void start(IIO &io, ITimer &timer)
{
    io.set_light(OnOff::Off);
    while (true)
    {
        wait_until_button_pressed(io);
        flash_until_button_released(io, timer);
    }
}

class PolledButtonBehavior
{
public:
    enum class States
    {
        NotPressed,
        BlinkOn,
        BlinkOff,
        ReleasedButton
    };
    PolledButtonBehavior(IIO &io, ITimer &timer) : io(io), timer(timer) {}
    void do_work()
    {
        // handle_state might perform multiple state transitions, so call
        // it repeatedly until it's done working.
        while (handle_state() == true)
        {
        }
    }

    // Return false when there is no more work to do
    bool handle_state()
    {
        switch (current_state)
        {
        case States::NotPressed:
            if (io.button_pressed())
            {
                current_state = States::BlinkOn;
                io.set_light(OnOff::On);
                timer.reset(1.0);
                return true;
            }
            break;
        case States::BlinkOn:
            if (io.button_released())
            {
                current_state = States::ReleasedButton;
                return true;
            }
            if (timer.expired())
            {
                io.set_light(OnOff::Off);
                timer.reset(1.0);
                current_state = States::BlinkOff;
                return true;
            }
            break;
        case States::BlinkOff:
            if (io.button_released())
            {
                current_state = States::ReleasedButton;
                return true;
            }
            if (timer.expired())
            {
                io.set_light(OnOff::On);
                timer.reset(1.0);
                current_state = States::BlinkOn;
                return true;
            }
            break;
        case States::ReleasedButton:
            io.set_light(OnOff::Off);
            current_state = States::NotPressed;
            return true;
            break;
        }
        return false;
    }

    States get_state() const
    {
        return current_state;
    }

protected:
    States current_state = States::NotPressed;
    IIO &io;
    ITimer &timer;
};

// Simple test to check equality of two numbers
TEST(StateMachine, FrameBehavior)
{
    TestIO io;
    TestTimer timer;
    PolledButtonBehavior behavior(io, timer);

    behavior.do_work();
    ASSERT_EQ(behavior.get_state(), PolledButtonBehavior::States::NotPressed);
    ASSERT_EQ(io.light_value, OnOff::Off);

    // Press the button
    io.button_pressed_value = true;
    behavior.do_work();
    // Light goes on immediately
    ASSERT_EQ(io.light_value, OnOff::On);
    ASSERT_EQ(behavior.get_state(), PolledButtonBehavior::States::BlinkOn);

    // Do work for a while and no change
    for (int i = 0; i < 100; ++i)
    {
        behavior.do_work();

        ASSERT_EQ(io.light_value, OnOff::On);
        ASSERT_EQ(behavior.get_state(), PolledButtonBehavior::States::BlinkOn);
    }

    // Let the timer expire and see that it transitions to blink off
    timer.expired_value = true;
    behavior.do_work();
    ASSERT_EQ(io.light_value, OnOff::Off);
    ASSERT_EQ(behavior.get_state(), PolledButtonBehavior::States::BlinkOff);

    // See that it transitions back to blink on when timer expired again
    timer.expired_value = true;
    behavior.do_work();
    ASSERT_EQ(io.light_value, OnOff::On);
    ASSERT_EQ(behavior.get_state(), PolledButtonBehavior::States::BlinkOn);

    // Release the button and it will double transition to not pressed
    io.button_pressed_value = false;
    behavior.do_work();
    ASSERT_EQ(io.light_value, OnOff::Off);
    ASSERT_EQ(behavior.get_state(), PolledButtonBehavior::States::NotPressed);
}