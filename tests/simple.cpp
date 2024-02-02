#include <gtest/gtest.h>

// Simple test to check equality of two numbers
TEST(NumberEqualityTest, HandlesIntegers) {
    int number1 = 5;
    int number2 = 5;

    ASSERT_EQ(number1, number2);
}