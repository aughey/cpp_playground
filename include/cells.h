#pragma once
#include <vector>
#include <optional>
#include <tuple>
#include <ranges>
#include <set>
#include <iostream>
#include <memory>

namespace cells
{
    // define XY as a tuple of two ints
    using XY = std::tuple<int, int>;

    enum class Direction
    {
        Up,
        Down,
        Left,
        Right
    };

    XY delta(const Direction &direction);

    enum class Cell
    {
        Space,
        Vertical,
        Horizontal,
        Slash,
        Backslash,
    };
    /// @brief Given a cell and a direction, what are all the possible next directions for this beam?
    /// @param self cell type entering
    /// @param entry direction of entering
    /// @return vector of directions
    std::vector<Direction> next_directions(const Cell &self, const Direction &entry);

    class Beam
    {
    public:
        Beam(XY location, Direction direction) : location(location), direction(direction) {}

        XY location;
        Direction direction;
    };

    class Grid
    {
    public:
        Grid(std::vector<std::vector<Cell>> cells) : cells_(cells) {}
        inline std::optional<Cell> at(int x, int y) const
        {
            if (x < 0 || y < 0 || x >= width() || y >= height())
            {
                return std::nullopt;
            }
            return cells_[y][x];
        }
        inline int width() const
        {
            if (cells_.size() == 0)
            {
                return 0;
            }
            return static_cast<int>(cells_[0].size());
        }
        inline int height() const
        {
            return static_cast<int>(cells_.size());
        }

    private:
        std::vector<std::vector<Cell>> cells_;
    };

    /// @brief Given a location, direction of entry, and cell, compute all next possible locations
    /// @param cur_location
    /// @param cur_direction
    /// @param cell
    /// @return
    inline auto next_possible_locations(const Cell &self, const Direction &cur_direction, const XY &cur_location)
    {
        auto dir_copy = std::make_shared<std::vector<Direction>>(next_directions(self, cur_direction));
      
        return *dir_copy | std::views::transform(
                                         [cur_location,dir_copy](const Direction &new_direction)
                                         {
                                             auto dx_dy = delta(new_direction);
                                             auto new_location = std::make_tuple(std::get<0>(cur_location) + std::get<0>(dx_dy), std::get<1>(cur_location) + std::get<1>(dx_dy));

                                             return std::make_tuple(new_location, new_direction);
                                         });
    }

    inline auto next_possible_beams(const Cell &self, const Direction &cur_direction, const XY &location)
    {
        return next_possible_locations(self, cur_direction, location) | std::views::transform(
                                                                            [](auto location_direction)
                                                                            {
                                                                                return Beam(std::get<0>(location_direction), std::get<1>(location_direction));
                                                                            });
    }

    using SpaceDirections = std::set<Direction>;

    class OccupyGrid
    {
    protected:
        std::vector<std::vector<SpaceDirections>> grid_;

    public:
        OccupyGrid(int width, int height)
        {
            grid_.resize(height);
            for (auto &row : grid_)
            {
                row.resize(width);
            }
        }
        bool visit(XY location, Direction direction)
        {
            auto [x, y] = location;

            auto &directions = grid_[y][x];
            if (directions.find(direction) != directions.end())
            {
                return false;
            }
            directions.insert(direction);
            return true;
        }
        std::size_t occupied_count()
        {
            std::size_t count = 0;
            for (auto &row : grid_)
            {
                for (auto &cell : row)
                {
                    if(!cell.empty()) {
                        count++;
                    }
                }
            }
            return count;
        }
    };

    std::size_t trace_grid(const Grid &grid, const XY &entry_location, const Direction &entry_direction);
}