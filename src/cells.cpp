#include "cells.h"
#include <stdexcept>
#include <iostream>
#include <sys/types.h>

namespace cells
{

    std::vector<Direction> next_directions(const Cell &self, const Direction &entry)
    {
        switch (self)
        {
        case Cell::Space:
            return {entry};
        case Cell::Vertical:
            switch (entry)
            {
            case Direction::Up:
                return {entry};
            case Direction::Down:
                return {entry};
            case Direction::Left:
                return {Direction::Up, Direction::Down};
            case Direction::Right:
                return {Direction::Up, Direction::Down};
            }
            break;
        case Cell::Horizontal:
            switch (entry)
            {
            case Direction::Up:
                return {Direction::Left, Direction::Right};
            case Direction::Down:
                return {Direction::Left, Direction::Right};
            case Direction::Left:
                return {entry};
            case Direction::Right:
                return {entry};
            }
            break;
        case Cell::Slash:
            switch (entry)
            {
            case Direction::Up:
                return {Direction::Right};
            case Direction::Down:
                return {Direction::Left};
            case Direction::Left:
                return {Direction::Down};
            case Direction::Right:
                return {Direction::Up};
            }
            break;
        case Cell::Backslash:
            switch (entry)
            {
            case Direction::Up:
                return {Direction::Left};
            case Direction::Down:
                return {Direction::Right};
            case Direction::Left:
                return {Direction::Up};
            case Direction::Right:
                return {Direction::Down};
            }
            break;
        }
        throw std::runtime_error("untested cell direction combination");
    }

    XY delta(const Direction &direction)
    {
        switch (direction)
        {
        case Direction::Up:
            return XY{0, -1};
        case Direction::Down:
            return XY{0, 1};
        case Direction::Left:
            return XY{-1, 0};
        case Direction::Right:
            return XY{1, 0};
        }
        throw std::runtime_error("untested direction");
    };

    std::size_t trace_grid(const Grid &grid, const XY &entry_location, const Direction &entry_direction)
    {
        // Active beams
        std::vector<Beam> beams = {Beam(entry_location, entry_direction)};

        // Keep track of where we've been
        auto occupy_grid = OccupyGrid(grid.width(), grid.height());

        // Keep ray tracing until we run out of beams
        while (!beams.empty())
        {
            auto beam = beams.back();
            beams.pop_back();

            auto opt_cell = grid.at(std::get<0>(beam.location), std::get<1>(beam.location));
            if (!opt_cell)
            {
                // We've gone off the grid
                continue;
            }
            // unwrap cell from optional
            auto cell = opt_cell.value();

            if (occupy_grid.visit(beam.location, beam.direction))
            {
                auto next_possible = next_possible_beams(cell, beam.direction, beam.location);
                // populate beams with the contents of next_possible
                beams.insert(beams.end(), next_possible.begin(), next_possible.end());
            }
        }
        return occupy_grid.occupied_count();
    }

}