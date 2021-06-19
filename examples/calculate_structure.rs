//! cargo run --example calculate_structure

/// Calulate the max proof length as long as the total amount isn't exceeded and the total transaction amount that can be reached with this options
// Example for row = 3; section_length = 3;
// Each "■" is a transaction and the arrows show where an output is consumed
// One section is "■ -> ■ -> ■"
// row 2:                                                                   ■ -----------------------------------------------> ■ -----------------------------------------------> ■ -----------------------------------------------⤴
// row 1:                ■ -------------> ■ -------------> ■ -------------⤴ ↪-------------> ■ -------------> ■ -------------⤴ ↪-------------> ■ -------------> ■ -------------⤴ ↪-------------> ■ -------------> ■ -------------⤴
// row 0:  ■ -> ■ -> ■ ⤴ ↪ ■ -> ■ -> ■ ⤴ ↪ ■ -> ■ -> ■ ⤴ ↪ ■ -> ■ -> ■ ⤴ ↪ ■ -> ■ -> ■ ⤴ ↪ ■ -> ■ -> ■ ⤴ ↪ ■ -> ■ -> ■ ⤴ ↪ ■ -> ■ -> ■ ⤴ ↪ ■ -> ■ -> ■ ⤴ ↪ ■ -> ■ -> ■ ⤴ ↪ ■ -> ■ -> ■ ⤴ ↪ ■ -> ■ -> ■ ⤴ ↪ ■ -> ■ -> ■ ⤴
// pos:    1    2    3   4  5    6    7   8  9    10   11  12  13  14   15 16  17   18   19  20 21   22   23  24 25   26   27  28 29   30   31  32 33   34   35  36 37   38    39 40  41  42    43 44 45    46   47 48 49    50  51  52
use tangleproof::inclusion_structure::*;

fn main() {
    // row == utxo_amount == required amount of Mi
    let rows = 2;
    // transactions from the same utxo until one of the next row will be used
    let section_length: u64 = 3;
    println!(
        "{:?}",
        get_previous_indexes_for_each_row_at_position(48, rows, section_length)
    );
    for i in 1..50 {
        println!(
            "Pos: {}, {:?}",
            i,
            get_previous_indexes_for_each_row_at_position(i, rows, section_length)
        );
    }
    println!(
        "Longest proof path (without more than section length txs before a new row starts): {}",
        (rows + 1) * section_length
    );
    println!(
        "Total proof transaction amount (without increasing max proof length): {}",
        get_row_starting_position(rows + 1, section_length) - 1
    );
    println!(
        "Starting position for last row: {}",
        get_row_starting_position(rows, section_length)
    );

    for row in 0..rows {
        println!(
            "Row {} starts at: {} interval length: {}",
            row,
            get_row_starting_position(row, section_length),
            get_row_length(row, section_length)
        );
    }

    for position in 0..32 {
        println!(
            "{} in row {}",
            position,
            get_row_for_position(position, rows, section_length)
        );
    }
}
