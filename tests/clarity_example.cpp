#include <gtest/gtest.h>

const int NUM_OTHER_VALUES = 16;

class Model
{
private:
    bool poweron = false;
    int state1 = 0;
    int state2 = 0;
    int state3 = 0;
    int other_values[NUM_OTHER_VALUES] = {0};

public:
    void turnPowerOn(bool on = true) { poweron = on; }
    void setStates(int value)
    {
        state1 = value;
        state2 = value;
        state3 = value;
    }
    void setOtherValues(int value)
    {
        for (auto &v : other_values)
        {
            v = value;
        }
    }

    // A state is valid if it is non-zero
    bool statesValid()
    {
        return state1 != 0 && state2 != 0 && state3 != 0;
    }

    bool poweredOn()
    {
        return poweron;
    }

    bool otherValuesNonZero() {
        for (const auto &v : other_values)
        {
            if (v == 0)
            {
                return false;
            }
        }
        return true;
    }

    // Requirement states that to be valid, the power must be on,
    // all states have a non-zero value, and all other_values are non-zero.
    bool isValid()
    {
        return poweredOn() && statesValid() && otherValuesNonZero();
    }

    // Requirement states that to be valid, the power must be on,
    // all states have a non-zero value, and all other_values are non-zero.
    bool isValidOld3()
    {
        if (!poweron || state1 == 0 || state2 == 0 || state3 == 0)
        {
            return false;
        }
        for (int i = 0; i < NUM_OTHER_VALUES; i++)
        {
            if (other_values[i] == 0)
            {
                return false;
            }
        }
        return true;
    }

    // Requirement states that to be valid, the power must be on,
    // all states have a non-zero value, and all other_values are non-zero.
    bool isValidOld2()
    {
        bool valid = true;
        if (!poweron || state1 == 0 || state2 == 0 || state3 == 0)
        {
            valid = false;
        }
        for (int i = 0; i < NUM_OTHER_VALUES; i++)
        {
            if (other_values[i] == 0)
            {
                valid = false;
                break;
            }
        }
        return valid;
    }

    // Requirement states that to be valid, the power must be on,
    // all states have a non-zero value, and all other_values are non-zero.
    bool isValidOld()
    {
        bool valid = true;
        if (poweron && state1 != 0 && state2 != 0 && state3 != 0)
        {
            for (int i = 0; i < NUM_OTHER_VALUES; i++)
            {
                if (other_values[i] == 0)
                {
                    valid = false;
                    break;
                }
            }
        }
        return valid;
    }
};

// Simple test to check equality of two numbers
TEST(ClarityExample, demo)
{
    auto model = Model();
    ASSERT_FALSE(model.isValid());
    model.turnPowerOn();
    ASSERT_FALSE(model.isValid());
    model.setStates(1);
    ASSERT_FALSE(model.isValid());
    model.setOtherValues(1);
    ASSERT_TRUE(model.isValid());
}