//! Grid coordinate utilities and basic Sudoku operations
//!
//! This module provides utility functions for converting between different
//! coordinate systems and performing basic Sudoku grid operations.

use crate::types::{BOX_SIZE, GRID_SIZE};

/// Converts row and column coordinates (0-8, 0-8) to a board index (0-80).
///
/// # Examples
/// ```
/// use sudoku_wasm::grid::coords_to_index;
/// assert_eq!(coords_to_index(0, 0), 0);   // Top-left corner
/// assert_eq!(coords_to_index(8, 8), 80);  // Bottom-right corner
/// assert_eq!(coords_to_index(4, 4), 40);  // Center cell
/// ```
pub fn coords_to_index(row: usize, col: usize) -> usize {
    row * GRID_SIZE + col
}

/// Converts a board index (0-80) to row and column coordinates (0-8, 0-8).
///
/// # Examples
/// ```
/// use sudoku_wasm::grid::index_to_coords;
/// assert_eq!(index_to_coords(0), (0, 0));   // Top-left corner
/// assert_eq!(index_to_coords(80), (8, 8));  // Bottom-right corner  
/// assert_eq!(index_to_coords(40), (4, 4));  // Center cell
/// ```
pub fn index_to_coords(index: usize) -> (usize, usize) {
    (index / GRID_SIZE, index % GRID_SIZE)
}

/// Calculates the 3x3 box index (0-8) for given row and column coordinates.
/// Boxes are numbered left-to-right, top-to-bottom.
///
/// # Examples
/// ```
/// use sudoku_wasm::grid::get_box_index;
/// assert_eq!(get_box_index(0, 0), 0);  // Top-left box
/// assert_eq!(get_box_index(8, 8), 8);  // Bottom-right box
/// assert_eq!(get_box_index(4, 4), 4);  // Center box
/// ```
///
/// # Arguments
/// * `row` - The row position (0-8)
/// * `col` - The column position (0-8)
///
/// # Returns
/// The box index (0-8)
pub fn get_box_index(row: usize, col: usize) -> usize {
    (row / BOX_SIZE) * BOX_SIZE + (col / BOX_SIZE)
}

/// Gets the starting coordinates of a 3x3 box given the box index
///
/// # Arguments
/// * `box_index` - The box index (0-8)
///
/// # Returns
/// A tuple (start_row, start_col) of the top-left corner of the box
pub fn get_box_start_coords(box_index: usize) -> (usize, usize) {
    let box_row = box_index / 3;
    let box_col = box_index % 3;
    (box_row * BOX_SIZE, box_col * BOX_SIZE)
}

/// Checks if placing a number at a given position violates Sudoku rules
///
/// This function validates that placing `num` at position (`row`, `col`)
/// doesn't conflict with existing numbers in the same row, column, or 3x3 box.
///
/// # Arguments
/// * `board` - The current board state (None for empty cells)
/// * `row` - The row position to check (0-8)
/// * `col` - The column position to check (0-8)
/// * `num` - The number to place (1-9)
///
/// # Returns
/// `true` if the placement is valid, `false` if it violates Sudoku rules
pub fn is_valid_placement(board: &[Option<u8>], row: usize, col: usize, num: u8) -> bool {
    // Check row for conflicts
    if has_conflict_in_row(board, row, num) {
        return false;
    }

    // Check column for conflicts
    if has_conflict_in_column(board, col, num) {
        return false;
    }

    // Check 3x3 box for conflicts
    if has_conflict_in_box(board, row, col, num) {
        return false;
    }

    true
}

/// Checks if a number already exists in the specified row
///
/// # Arguments
/// * `board` - The current board state
/// * `row` - The row to check (0-8)
/// * `num` - The number to look for (1-9)
///
/// # Returns
/// `true` if the number exists in the row
fn has_conflict_in_row(board: &[Option<u8>], row: usize, num: u8) -> bool {
    for col in 0..GRID_SIZE {
        if board[coords_to_index(row, col)] == Some(num) {
            return true;
        }
    }
    false
}

/// Checks if a number already exists in the specified column
///
/// # Arguments
/// * `board` - The current board state
/// * `col` - The column to check (0-8)
/// * `num` - The number to look for (1-9)
///
/// # Returns
/// `true` if the number exists in the column
fn has_conflict_in_column(board: &[Option<u8>], col: usize, num: u8) -> bool {
    for row in 0..GRID_SIZE {
        if board[coords_to_index(row, col)] == Some(num) {
            return true;
        }
    }
    false
}

/// Checks if a number already exists in the 3x3 box containing the given position
///
/// # Arguments
/// * `board` - The current board state
/// * `row` - The row position within the box (0-8)
/// * `col` - The column position within the box (0-8)
/// * `num` - The number to look for (1-9)
///
/// # Returns
/// `true` if the number exists in the box
fn has_conflict_in_box(board: &[Option<u8>], row: usize, col: usize, num: u8) -> bool {
    let box_start_row = (row / BOX_SIZE) * BOX_SIZE;
    let box_start_col = (col / BOX_SIZE) * BOX_SIZE;

    for r in box_start_row..box_start_row + BOX_SIZE {
        for c in box_start_col..box_start_col + BOX_SIZE {
            if board[coords_to_index(r, c)] == Some(num) {
                return true;
            }
        }
    }
    false
}

/// Gets all cell indices that belong to the same row as the given cell
///
/// # Arguments
/// * `index` - The reference cell index (0-80)
///
/// # Returns
/// A vector of indices representing all cells in the same row
pub fn get_row_indices(index: usize) -> Vec<usize> {
    let (row, _) = index_to_coords(index);
    (0..GRID_SIZE)
        .map(|col| coords_to_index(row, col))
        .collect()
}

/// Gets all cell indices that belong to the same column as the given cell
///
/// # Arguments
/// * `index` - The reference cell index (0-80)
///
/// # Returns
/// A vector of indices representing all cells in the same column
pub fn get_column_indices(index: usize) -> Vec<usize> {
    let (_, col) = index_to_coords(index);
    (0..GRID_SIZE)
        .map(|row| coords_to_index(row, col))
        .collect()
}

/// Gets all cell indices that belong to the same 3x3 box as the given cell
///
/// # Arguments
/// * `index` - The reference cell index (0-80)
///
/// # Returns
/// A vector of indices representing all cells in the same box
pub fn get_box_indices(index: usize) -> Vec<usize> {
    let (row, col) = index_to_coords(index);
    let box_start_row = (row / BOX_SIZE) * BOX_SIZE;
    let box_start_col = (col / BOX_SIZE) * BOX_SIZE;

    let mut indices = Vec::new();
    for r in box_start_row..box_start_row + BOX_SIZE {
        for c in box_start_col..box_start_col + BOX_SIZE {
            indices.push(coords_to_index(r, c));
        }
    }
    indices
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_coords_to_index() {
        assert_eq!(coords_to_index(0, 0), 0);
        assert_eq!(coords_to_index(0, 8), 8);
        assert_eq!(coords_to_index(8, 0), 72);
        assert_eq!(coords_to_index(8, 8), 80);
        assert_eq!(coords_to_index(4, 4), 40);
    }

    #[test]
    fn test_index_to_coords() {
        assert_eq!(index_to_coords(0), (0, 0));
        assert_eq!(index_to_coords(8), (0, 8));
        assert_eq!(index_to_coords(72), (8, 0));
        assert_eq!(index_to_coords(80), (8, 8));
        assert_eq!(index_to_coords(40), (4, 4));
    }

    #[test]
    fn test_get_box_index() {
        assert_eq!(get_box_index(0, 0), 0); // Top-left box
        assert_eq!(get_box_index(0, 4), 1); // Top-center box
        assert_eq!(get_box_index(0, 8), 2); // Top-right box
        assert_eq!(get_box_index(4, 0), 3); // Middle-left box
        assert_eq!(get_box_index(4, 4), 4); // Center box
        assert_eq!(get_box_index(8, 8), 8); // Bottom-right box
    }
}
