use super::grid::{GridTile, LevelGrid, DIAGONAL_DIRECTIONS, DIRECTIONS, LEVEL_SIZE};
use rand::prelude::*;

type CursorPosition = (isize, isize, isize);

fn generate_height_limits(rng: &mut ThreadRng) -> [[u8; LEVEL_SIZE.1]; LEVEL_SIZE.0] {
    let size = (
        rng.gen_range(LEVEL_SIZE.0 / 2..LEVEL_SIZE.0 * 3 / 4),
        rng.gen_range(LEVEL_SIZE.1 / 2..LEVEL_SIZE.1 * 3 / 4),
    );

    let position = (
        rng.gen_range(0..LEVEL_SIZE.0 - size.0),
        rng.gen_range(0..LEVEL_SIZE.1 - size.1),
    );

    let inner_size = (
        rng.gen_range(size.0 / 2..size.0 * 3 / 4),
        rng.gen_range(size.1 / 2..size.1 * 3 / 4),
    );

    let inner_position = (
        rng.gen_range(position.0..position.0 + size.0 - inner_size.0),
        rng.gen_range(position.1..position.1 + size.1 - inner_size.1),
    );

    let mut out = [[0; LEVEL_SIZE.1]; LEVEL_SIZE.0];

    for x in 0..LEVEL_SIZE.0 {
        for y in 0..LEVEL_SIZE.1 {
            out[x][y] = if inner_position.0 <= x
                && x < inner_position.0 + inner_size.0
                && inner_position.1 <= y
                && y < inner_position.1 + inner_size.1
            {
                2
            } else if position.0 <= x
                && x < position.0 + size.0
                && position.1 <= y
                && y < position.1 + size.1
            {
                1
            } else {
                0
            }
        }
    }

    return out;
}

fn generate_options(
    grid: &LevelGrid,
    cursor_position: CursorPosition,
) -> Vec<(GridTile, (isize, isize), Option<(isize, isize)>)> {
    let mut options = Vec::new();
    let mut expanded_diagonals = 0;

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

        if grid.get(
            offset_tile_position.0,
            offset_tile_position.1,
            cursor_position.2,
        ) == GridTile::Empty
            && grid.is_valid_height(
                offset_tile_position.0,
                offset_tile_position.1,
                cursor_position.2,
            )
        {
            options.push((GridTile::Floor, offset_tile_position, None));

            let expanded_grid_tile = (
                (offset_tile_position.0 + cardinal_direction.0),
                (offset_tile_position.1 + cardinal_direction.1),
            );

            if DIRECTIONS.iter().any(|dir| {
                grid.get(
                    offset_tile_position.0 + dir.0,
                    offset_tile_position.1 + dir.1,
                    cursor_position.2,
                )
                .is_stair_tile()
            }) {
                continue;
            }

            if grid.get(
                offset_tile_position.0,
                offset_tile_position.1,
                cursor_position.2 - 1,
            ) == GridTile::Empty
                && grid.get(
                    expanded_grid_tile.0,
                    expanded_grid_tile.1,
                    cursor_position.2 - 1,
                ) == GridTile::Empty
                && grid.is_valid_height(
                    expanded_grid_tile.0,
                    expanded_grid_tile.1,
                    cursor_position.2 - 1,
                )
            {
                // Prevent the same stair spawning diagonal from itself, which looks ugly
                if !DIAGONAL_DIRECTIONS.iter().any(|(a, b)| {
                    grid.get(
                        offset_tile_position.0 + a,
                        offset_tile_position.1 + b,
                        cursor_position.2,
                    ) == cardinal_direction.2
                }) {
                    options.push((
                        cardinal_direction.2,
                        offset_tile_position,
                        Some(expanded_grid_tile),
                    ))
                } else {
                    expanded_diagonals += 1;
                }
            }

            if grid.get(
                offset_tile_position.0,
                offset_tile_position.1,
                cursor_position.2 + 1,
            ) == GridTile::Empty
                && grid.get(
                    expanded_grid_tile.0,
                    expanded_grid_tile.1,
                    cursor_position.2 + 1,
                ) == GridTile::Empty
                && grid.is_valid_height(
                    expanded_grid_tile.0,
                    expanded_grid_tile.1,
                    cursor_position.2 + 1,
                )
            {
                // Prevent the same stair spawning diagonal from itself, which looks ugly
                if !DIAGONAL_DIRECTIONS.iter().any(|(a, b)| {
                    grid.get(
                        offset_tile_position.0 + a,
                        offset_tile_position.1 + b,
                        cursor_position.2,
                    ) == cardinal_direction.3
                }) {
                    options.push((
                        cardinal_direction.3,
                        offset_tile_position,
                        Some(expanded_grid_tile),
                    ))
                } else {
                    expanded_diagonals += 1;
                }
            }
        }
    }

    if expanded_diagonals > 0 {
        println!("Removed expanded diagonals: {expanded_diagonals} at {cursor_position:?}");
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
                    grid.get(new_tile_position.0, new_tile_position.1, cursor_position.2),
                    GridTile::Empty
                );
                grid.set(
                    new_tile_position.0,
                    new_tile_position.1,
                    cursor_position.2,
                    *new_tile_type,
                );
                cursor_position.0 = new_tile_position.0;
                cursor_position.1 = new_tile_position.1;
                if new_tile_type.is_stair_tile() {
                    if new_tile_type.is_bottom_stair_tile() {
                        cursor_position.2 += 1;
                    } else if new_tile_type.is_top_stair_tile() {
                        cursor_position.2 -= 1;
                    }
                    println!("# {:?}", cursor_position);
                    grid.set(
                        new_tile_position.0,
                        new_tile_position.1,
                        cursor_position.2,
                        new_tile_type.get_opposite_stair_tile().unwrap(),
                    );

                    let new_new_tile_position = new_new_tile_position.unwrap();
                    cursor_position.0 = new_new_tile_position.0;
                    cursor_position.1 = new_new_tile_position.1;

                    grid.set(
                        cursor_position.0,
                        cursor_position.1,
                        cursor_position.2 as isize,
                        GridTile::Floor,
                    );
                }
            }
            None => break,
        }
    }
}

fn is_dead_end(grid: &LevelGrid, x: isize, y: isize, z: isize) -> bool {
    let tile = grid.get(x, y, z);

    let mut nr = 0;
    if tile.is_bottom_stair_tile() && grid.get(x, y, z + 1).is_top_stair_tile() {
        nr += 1;
    }

    if tile.is_top_stair_tile() && grid.get(x, y, z - 1).is_bottom_stair_tile() {
        nr += 1;
    }

    for direction in DIRECTIONS {
        if tile.can_access(direction)
            && grid
                .get(x + direction.0, y + direction.1, z)
                .can_access((-direction.0, -direction.1))
        {
            nr += 1
        }
    }

    return nr == 1;
}

fn remove_dead_ends(grid: &mut LevelGrid) {
    let mut nrof_dead_ends_removed = 0;
    let mut dead_ends = vec![];

    for x in 0..LEVEL_SIZE.0 as isize {
        for y in 0..LEVEL_SIZE.1 as isize {
            for z in 0..LEVEL_SIZE.2 as isize {
                if is_dead_end(grid, x, y, z) {
                    dead_ends.push((x, y, z));
                }
            }
        }
    }

    while dead_ends.len() > 0 {
        let mut new_dead_ends = vec![];

        for (x, y, z) in dead_ends {
            grid.set(x, y, z, GridTile::Empty);
            nrof_dead_ends_removed += 1;

            for (x, y, z) in [
                (x + 1, y, z),
                (x - 1, y, z),
                (x, y + 1, z),
                (x, y - 1, z),
                (x, y, z - 1),
                (x, y, z + 1),
            ] {
                if is_dead_end(grid, x, y, z) {
                    new_dead_ends.push((x, y, z));
                }
            }
        }

        dead_ends = new_dead_ends;
    }

    println!("Removed {nrof_dead_ends_removed} dead ends");
}

pub(super) fn generate_level_grid() -> Box<LevelGrid> {
    let mut grid = LevelGrid::new(generate_height_limits(&mut rand::thread_rng()));

    let cursor_position: CursorPosition = (6, 6, grid.1[6][6] as isize);
    grid.set(
        cursor_position.0,
        cursor_position.1,
        cursor_position.2,
        GridTile::Floor,
    );

    generate_single_path(&mut grid, cursor_position);
    generate_single_path(&mut grid, cursor_position);

    remove_dead_ends(&mut grid);

    return Box::new(grid);
}
