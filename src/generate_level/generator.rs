use super::{get_grid_at, GridTile, LevelGrid, LEVEL_SIZE};
use rand::prelude::*;

type CursorPosition = (isize, isize, isize);

fn generate_options(
    grid: &LevelGrid,
    cursor_position: CursorPosition,
) -> Vec<(GridTile, (isize, isize), Option<(isize, isize)>)> {
    let mut options = Vec::new();

    for cardinal_direction in [
        (-1, 0, GridTile::StairsWestTop, GridTile::StairsEastBottom),
        (0, -1, GridTile::StairsNorthTop, GridTile::StairsSouthBottom),
        (1, 0, GridTile::StairsEastTop, GridTile::StairsWestBottom),
        (0, 1, GridTile::StairsSouthTop, GridTile::StairsNorthBottom),
    ] {
        let offset_tile_position = (
            cardinal_direction.0 + cursor_position.0,
            cardinal_direction.1 + cursor_position.1,
        );

        if get_grid_at(
            &grid,
            offset_tile_position.0,
            offset_tile_position.1,
            cursor_position.2,
        ) == GridTile::Empty
        {
            options.push((GridTile::Floor, offset_tile_position, None));

            if offset_tile_position.0 + cardinal_direction.0 > 0
                && offset_tile_position.1 + cardinal_direction.1 > 0
                && offset_tile_position.0 + cardinal_direction.0 < grid.len() as isize
                && offset_tile_position.1 + cardinal_direction.1 < grid[0].len() as isize
            {
                let expanded_grid_tile = (
                    (offset_tile_position.0 + cardinal_direction.0),
                    (offset_tile_position.1 + cardinal_direction.1),
                );

                if get_grid_at(
                    &grid,
                    offset_tile_position.0,
                    offset_tile_position.1,
                    cursor_position.2 - 1,
                ) == GridTile::Empty
                    && get_grid_at(
                        &grid,
                        expanded_grid_tile.0,
                        expanded_grid_tile.1,
                        cursor_position.2 - 1,
                    ) == GridTile::Empty
                {
                    options.push((
                        cardinal_direction.2,
                        offset_tile_position,
                        Some(expanded_grid_tile),
                    ))
                }

                if get_grid_at(
                    &grid,
                    offset_tile_position.0,
                    offset_tile_position.1,
                    cursor_position.2 + 1,
                ) == GridTile::Empty
                    && get_grid_at(
                        &grid,
                        expanded_grid_tile.0,
                        expanded_grid_tile.1,
                        cursor_position.2 + 1,
                    ) == GridTile::Empty
                {
                    options.push((
                        cardinal_direction.3,
                        offset_tile_position,
                        Some(expanded_grid_tile),
                    ))
                }
            }
        }
    }

    return options;
}

fn generate_single_path(grid: &mut LevelGrid, mut cursor_position: CursorPosition) {
    let mut rng = rand::thread_rng();

    loop {
        let options = generate_options(&grid, cursor_position);

        println!("> {:?}", cursor_position);
        match options.choose(&mut rng) {
            Some((new_tile_type, new_tile_position, new_new_tile_position)) => {
                // get_grid_at(&grid, new_tile_position.0, new_tile_position.1, cursor_position.2);
                assert_eq!(
                    get_grid_at(
                        &grid,
                        new_tile_position.0,
                        new_tile_position.1,
                        cursor_position.2
                    ),
                    GridTile::Empty
                );
                grid[new_tile_position.0 as usize][new_tile_position.1 as usize]
                    [cursor_position.2 as usize] = *new_tile_type;
                cursor_position.0 = new_tile_position.0;
                cursor_position.1 = new_tile_position.1;
                if new_tile_type.is_stair_tile() {
                    if new_tile_type.is_bottom_stair_tile() {
                        cursor_position.2 += 1;
                    } else if new_tile_type.is_top_stair_tile() {
                        cursor_position.2 -= 1;
                    }
                    println!("# {:?}", cursor_position);
                    grid[new_tile_position.0 as usize][new_tile_position.1 as usize]
                        [cursor_position.2 as usize] =
                        new_tile_type.get_opposite_stair_tile().unwrap();

                    let new_new_tile_position = new_new_tile_position.unwrap();
                    cursor_position.0 = new_new_tile_position.0;
                    cursor_position.1 = new_new_tile_position.1;

                    grid[cursor_position.0 as usize][cursor_position.1 as usize]
                        [cursor_position.2 as usize] = GridTile::Floor;
                }
            }
            None => break,
        }
    }
}

pub(super) fn generate_level_grid() -> Box<LevelGrid> {
    let mut grid = [[[GridTile::Empty; LEVEL_SIZE.2]; LEVEL_SIZE.1]; LEVEL_SIZE.0];

    let cursor_position: CursorPosition = (6, 6, 1);
    grid[cursor_position.0 as usize][cursor_position.1 as usize][cursor_position.2 as usize] =
        GridTile::Floor;

    generate_single_path(&mut grid, cursor_position);
    generate_single_path(&mut grid, cursor_position);

    return Box::new(grid);
}