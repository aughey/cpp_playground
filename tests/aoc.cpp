#include <gtest/gtest.h>
#include <iostream>
#include <string>
#include <ranges>
#include "lib.h"

TEST(AoC, Part1) {
    auto data = std::string(lib::sample_data());
    auto cells = lib::lines_to_grid(data);

    cells::XY start = {0,0};
    cells::Direction direction = cells::Direction::Right;
    auto count = cells::trace_grid(cells, start,direction);

    ASSERT_EQ(count, 46);
}
