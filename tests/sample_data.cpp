#include <gtest/gtest.h>
#include "lib.h"

TEST(SampleData, ItExists) {
    auto data = lib::sample_data();
    ASSERT_TRUE(data);
    ASSERT_EQ(data[0], '.');
    ASSERT_EQ(data[1], '|');
}


