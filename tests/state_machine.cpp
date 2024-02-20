#pragma GCC diagnostic ignored "-Wunused-parameter"

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

class IO
{
public:
    void set_light(OnOff on_or_off)
    {
        // Set light
    }

    bool button_pressed()
    {
        return true;
    }
    bool button_released()
    {
        return true;
    }
};

class Timer
{
public:
    Timer(double seconds) {}
    bool expired() const
    {
        return false;
    }
};

/// Returns RELEASED if button released, TIMER if timer expired
FlashResult flash_stage(OnOff on_or_off, const Timer &timer, IO &io)
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
void flash_until_release(IO &io)
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
void wait_until_pressed(IO &io)
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