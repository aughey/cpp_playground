#include "lib.h"
#include <ranges>

static const char *sample_data = R"(.|...\....
|.-.\.....
.....|-...
........|.
..........
.........\
..../.\\..
.-.-/..|..
.|....-|.\
..//.|....)";

namespace lib
{
    const char *sample_data()
    {
        return ::sample_data;
    }
    cells::Cell char_to_cell(char c)
    {
        switch (c)
        {
        case '.':
            return cells::Cell::Space;
        case '|':
            return cells::Cell::Vertical;
        case '-':
            return cells::Cell::Horizontal;
        case '/':
            return cells::Cell::Slash;
        case '\\':
            return cells::Cell::Backslash;
        default:
            throw std::runtime_error("Invalid character in line");
        }
    }
    auto line_to_cells(auto line)
    {
        return line | std::views::transform(char_to_cell);
    }
    auto split_line_on_newline(const std::string &line)
    {
        return line | std::views::split('\n');
    }
    cells::Grid lines_to_grid(const std::string &lines)
    {
        std::vector<std::vector<cells::Cell>> data;
        for (const auto &line : split_line_on_newline(lines))
        {
            std::vector<cells::Cell> row;
            auto cells = line_to_cells(line);
            row.insert(row.end(), cells.begin(), cells.end());
            data.push_back(row);
        }
        return cells::Grid(data);
    }
}
