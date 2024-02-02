#include <gtest/gtest.h>
#include <iostream>
#include <string>
#include <ranges>
#include "lib.h"

TEST(CellReading, LoadingWorks) {
    auto data = std::string(lib::sample_data());
    auto cells = lib::lines_to_grid(data);

    ASSERT_EQ(cells.width(), 10);
    ASSERT_EQ(cells.height(), 10);
    ASSERT_EQ(cells.at(0, 0), cells::Cell::Space);
    ASSERT_EQ(cells.at(1, 0), cells::Cell::Vertical);
    ASSERT_EQ(cells.at(2, 1), cells::Cell::Horizontal);
}

TEST(CellReading, OutOfRangeFails) {
    auto data = std::string(lib::sample_data());
    auto cells = lib::lines_to_grid(data);

    ASSERT_EQ(cells.width(), 10);
    ASSERT_EQ(cells.height(), 10);
    ASSERT_EQ(cells.at(0, 0), cells::Cell::Space);
    ASSERT_TRUE(cells.at(1, 0));
    ASSERT_FALSE(cells.at(-1, 0));
}