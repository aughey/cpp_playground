#include <gtest/gtest.h>
#include "cells.h"

TEST(NextDirection, Expected) {
    auto cell = cells::Cell::Vertical;
    auto entry = cells::Direction::Up;
    auto directions = cells::next_directions(cell, entry);
    ASSERT_EQ(directions.size(), 1);
    ASSERT_EQ(directions[0], cells::Direction::Up);
}
