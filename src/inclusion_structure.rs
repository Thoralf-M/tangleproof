/// Calulate the max proof length as long as the total amount isn't exceeded and the total transaction amount that can be reached with this options

// Example for row = 3; section_length = 3;
// Each "■" is a transaction and the arrows show where an output is consumed
// One section is "■ -> ■ -> ■"
// row 2:                                                                   ■ -----------------------------------------------> ■ -----------------------------------------------> ■ -----------------------------------------------⤴
// row 1:                ■ -------------> ■ -------------> ■ -------------⤴ ↪-------------> ■ -------------> ■ -------------⤴ ↪-------------> ■ -------------> ■ -------------⤴ ↪-------------> ■ -------------> ■ -------------⤴
// row 0:  ■ -> ■ -> ■ ⤴ ↪ ■ -> ■ -> ■ ⤴ ↪ ■ -> ■ -> ■ ⤴ ↪ ■ -> ■ -> ■ ⤴ ↪ ■ -> ■ -> ■ ⤴ ↪ ■ -> ■ -> ■ ⤴ ↪ ■ -> ■ -> ■ ⤴ ↪ ■ -> ■ -> ■ ⤴ ↪ ■ -> ■ -> ■ ⤴ ↪ ■ -> ■ -> ■ ⤴ ↪ ■ -> ■ -> ■ ⤴ ↪ ■ -> ■ -> ■ ⤴ ↪ ■ -> ■ -> ■ ⤴
// pos:    1    2    3   4  5    6    7   8  9    10   11  12  13  14   15 16  17   18   19  20 21   22   23  24 25   26   27  28 29   30   31  32 33   34   35  36 37   38    39 40  41  42    43 44 45    46   47 48 49    50  51  52

pub fn get_previous_indexes_for_each_row_at_position(
    position: u64,
    rows: u64,
    section_length: u64,
) -> Vec<(u64, u64)> {
    if position == 0 {
        return vec![];
    }
    // (position, row), with position for row 0 inserted
    let mut previous_indexes: Vec<(u64, u64)> = vec![(position - 1, 0)];

    for row in (1..rows).rev() {
        let amount_before_row_starts = get_row_starting_position(row, section_length);
        if position > amount_before_row_starts
            && position % (section_length + 1) == 0
            && get_row_for_position(position, rows, section_length) >= row
        {
            previous_indexes.push((position - get_row_section_length(row, section_length), row))
        }
    }

    previous_indexes
}

// Get highest row for a position
pub fn get_row_for_position(position: u64, row: u64, section_length: u64) -> u64 {
    let mut in_row = 0;
    for row in 1..row + 1 {
        let interval = get_row_length(row, section_length);
        let amount_before_row_starts = get_row_starting_position(row, section_length);
        if position >= amount_before_row_starts
            && (position - amount_before_row_starts) % (interval / section_length) == 0
        {
            in_row = row;
        };
    }
    in_row
}

pub fn get_row_length(row: u64, section_length: u64) -> u64 {
    (section_length + 1) * section_length.pow(row as u32)
}

pub fn get_row_section_length(row: u64, section_length: u64) -> u64 {
    get_row_length(row, section_length) / section_length
}

pub fn get_row_starting_position(row: u64, section_length: u64) -> u64 {
    let mut position = 0;
    for o in 0..row {
        position += (section_length + 1) * section_length.pow(o as u32);
    }
    position
}

// Get children_path with (position, row)
pub fn get_path(
    position: u64,
    current_max_position: u64,
    row: u64,
    section_length: u64,
) -> Vec<(u64, u64)> {
    // let now = std::time::Instant::now();
    let mut children_path = Vec::new();
    let mut position_index = position;
    while position_index <= current_max_position {
        // println!("position_index {}", position_index);
        let row = get_row_for_position(position_index, row, section_length);
        if children_path.is_empty() {
            children_path.push((position_index, row));
        } else {
            // safe to unwrap since we checked that it's not empty
            if children_path.last().unwrap().1 <= row {
                children_path.push((position_index, row));
            }
        }
        // increase index until we reach the next block in the current row
        if position_index % (section_length + 1) == 0 {
            // safe to unwrap since we checked that it's not empty
            position_index +=
                get_row_length(children_path.last().unwrap().1, section_length) / section_length;
        } else {
            // currently in row 0, increase index until we get to one with a higher row
            position_index += 1;
        }
    }
    // println!("Fast took: {:.2?}", now.elapsed());
    // // slow, but maybe easier to understand
    // let mut children_path_ = Vec::new();
    // for position_index in position..current_max_position {
    //     // println!("position_index {}", position_index);
    //     // increase position to next interval of last row
    //     let row = get_row_for_position(position_index, row, section_length);
    //     if children_path_.is_empty() {
    //         children_path_.push((position_index, row));
    //     } else {
    //         if children_path_.last().unwrap().1 <= row {
    //             children_path_.push((position_index, row));
    //         }
    //     }
    // }
    // assert_eq!(children_path, children_path_);

    children_path
}
