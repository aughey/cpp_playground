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

class IIO
{
public:
    virtual void set_light(OnOff on_or_off) = 0;
    virtual bool button_pressed() = 0;
    virtual bool button_released() = 0;
};

class IO : public IIO
{
public:
    void set_light(OnOff on_or_off) override
    {
        // Set the light
    }
    bool button_pressed() override
    {
        // Return true if the button is pressed
        return true;
    }
    bool button_released() override
    {
        // Return true if the button is released
        return true;
    }
};

class ITimer
{
public:
    virtual void reset(double seconds) = 0;
    virtual bool expired() const = 0;
};

class Timer : public ITimer
{
public:
    Timer() {}
    Timer(double seconds) {}
    void reset(double seconds) override
    {
        // Reset the timer
    }
    bool expired() const override
    {
        // Return true if the timer has expired
        return true;
    }
};

/// Returns RELEASED if button released, ITimer if ITimer expired
FlashResult flash_stage(OnOff on_or_off, const ITimer &timer, IIO &io)
{
    io.set_light(on_or_off);

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

void flash_until_release(IIO &io)
{
    // Flashing
    while (true)
    {
        if (FlashResult::Released == flash_stage(OnOff::On, Timer(1.0), io))
        {
            break;
        }
        if (FlashResult::Released == flash_stage(OnOff::Off, Timer(1.0), io))
        {
            break;
        }
    }
}

void wait_until_pressed(IIO &io)
{
    while (true)
    {
        if (io.button_pressed())
        {
            break;
        }
    }
}

void start()
{
    IO io;
    while (true)
    {
        io.set_light(OnOff::Off);
        wait_until_pressed(io);
        flash_until_release(io);
    }
}

class PolledButtonBehavior
{
public:
    PolledButtonBehavior(IIO &io, ITimer &timer) : io(io), timer(timer) {}
    void do_work()
    {
        while (handle_state() == true)
        {
        }
    }

    bool handle_state()
    {
        switch (current_state)
        {
        case States::NotPressed:
            if (io.button_pressed())
            {
                current_state = States::BlinkOn;
                io.set_light(OnOff::Off);
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
                current_state = States::BlinkOff;
                return true;
            }
            break;
        case States::ReleasedButton:
            io.set_light(OnOff::Off);
            current_state = States::NotPressed;
            break;
        }
    }

protected:
    enum class States
    {
        NotPressed,
        BlinkOn,
        BlinkOff,
        ReleasedButton
    };
    States current_state = States::NotPressed;
    IIO &io;
    ITimer &timer;
};

// Simple test to check equality of two numbers
TEST(StateMachine, FrameBehavior)
{
    ASSERT_EQ(1, 1);
}