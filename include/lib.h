#pragma once
#include <string>
#include <cells.h>

namespace lib
{
    const char *sample_data();
    cells::Grid lines_to_grid(const std::string &lines);
}