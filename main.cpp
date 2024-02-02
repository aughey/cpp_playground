#include <iostream>
#include <fstream>
#include <sstream>
#include <tuple>
#include <ranges>
#include "lib.h"
#include "cells.h"

auto file_lines(const char *filename) {
    std::ifstream file(filename);
    // use ranges to split this into lines
    return std::ranges::istream_view<std::string>(file);
}

std::string read_file(const char *filename)
{
    std::ifstream file(filename);
    std::stringstream buffer;
    buffer << file.rdbuf();
    return buffer.str();
}

auto split_lines(const std::string &contents) {
    return contents | std::ranges::views::split('\n') | std::ranges::views::transform([](auto &&rng) {
        return std::string(rng.begin(), rng.end());
    });
}

auto split_lines_collected(const std::string &contents) {
    std::vector<std::string> lines;
    for(auto &&rng : contents | std::ranges::views::split('\n')) {
        lines.push_back(std::string(rng.begin(), rng.end()));
    }
    return lines;
}

auto problem_lines(const char *filename) {
    auto contents = read_file(filename);
    return split_lines(contents);
}

int main()
{
    auto filename = "problem.txt";
    // std::ifstream file(filename);
    // // use ranges to split this into lines
    // auto lines =  file | std::ranges::views::split("\n");
    // for(auto line : lines) {
    //     std::cout << "LINE: " << line << "\n";
    // }
    // return 0;
    auto contents = read_file(filename);
    for(auto line : split_lines(contents)) {
        std::cout << "LINE: " << line << "\n";
    }

    for(auto line : problem_lines(filename)) {
        std::cout << "Problem LINE: " << line << "\n";
    }

    return 0;

    // read file to string
    std::string file_contents;
    {
        std::ifstream file("problem.txt");
        std::string str;
        while (std::getline(file, str))
        {
            if (!file_contents.empty())
            {
                file_contents.push_back('\n');
            }
            file_contents += str;
        }
    }

    auto grid = lib::lines_to_grid(file_contents);

    cells::XY start = {0, 0};
    cells::Direction direction = cells::Direction::Right;
    auto count = cells::trace_grid(grid, start, direction);

    std::cout << "Part 1: " << count << "\n";

    return 0;
}
