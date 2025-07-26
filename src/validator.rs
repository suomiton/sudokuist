//! Board validation and solving utilities
//!
//! This module provides functions for validating Sudoku boards,
//! checking for conflicts, and solving puzzles using backtracking.

use crate::grid::{index_to_coords, is_valid_placement};
use crate::types::{ValidationResult, BOARD_SIZE};

/// Validates the current board state and identifies any rule violations
///
/// Checks each filled cell to ensure it doesn't conflict with Sudoku rules
/// in its row, column, or 3x3 box. Also determines if the board is complete.
///
/// # Arguments
/// * `board` - The current board state with Some(num) for filled cells, None for empty
///
/// # Returns
/// A `ValidationResult` containing invalid cell indices and completion status
pub fn validate_board(board: &[Option<u8>]) -> ValidationResult {
    let mut invalid_indices = Vec::new();
    let mut is_complete = true;

    for index in 0..BOARD_SIZE {
        let (_row, _col) = index_to_coords(index);

        match board[index] {
            None => {
                // Empty cell means board is not complete
                is_complete = false;
            }
            Some(num) => {
                // Validate that this number placement is legal
                if !is_placement_valid_at_index(board, index, num) {
                    invalid_indices.push(index);
                }
            }
        }
    }

    ValidationResult {
        invalid_indices: invalid_indices.clone(),
        is_complete: is_complete && invalid_indices.is_empty(),
    }
}

/// Checks if a number placement at a specific index is valid
///
/// Temporarily removes the cell value and checks if placing the number
/// would be valid according to Sudoku rules.
///
/// # Arguments
/// * `board` - The current board state
/// * `index` - The cell index to check (0-80)
/// * `num` - The number to validate (1-9)
///
/// # Returns
/// `true` if the placement is valid
fn is_placement_valid_at_index(board: &[Option<u8>], index: usize, num: u8) -> bool {
    let (row, col) = index_to_coords(index);

    // Create a temporary board with this cell empty to test placement
    let mut temp_board = board.to_vec();
    temp_board[index] = None;

    is_valid_placement(&temp_board, row, col, num)
}

/// Checks if a puzzle has a unique solution
///
/// Uses backtracking to attempt to solve the puzzle. This is a simplified
/// uniqueness check that verifies the puzzle is solvable.
///
/// # Arguments
/// * `board` - The puzzle board to check
///
/// # Returns
/// `true` if the puzzle has a solution (uniqueness not fully verified)
///
/// # Note
/// This is a simplified implementation. A full uniqueness check would
/// need to count all possible solutions, which is computationally expensive.
pub fn has_unique_solution(board: &[Option<u8>]) -> bool {
    let mut test_board = board.to_vec();
    solve_board(&mut test_board)
}

/// Solves a Sudoku board using backtracking algorithm
///
/// This is a complete backtracking solver that finds any valid solution
/// to the given puzzle. It modifies the board in-place.
///
/// # Arguments
/// * `board` - Mutable reference to the board to solve
///
/// # Returns
/// `true` if a solution was found, `false` if unsolvable
pub fn solve_board(board: &mut [Option<u8>]) -> bool {
    // Find the next empty cell
    let empty_cell_index = find_next_empty_cell(board);

    match empty_cell_index {
        None => {
            // No empty cells means the board is complete
            true
        }
        Some(index) => {
            let (row, col) = index_to_coords(index);

            // Try each number 1-9 in this position
            for num in 1..=9 {
                if is_valid_placement(board, row, col, num) {
                    // Place the number
                    board[index] = Some(num);

                    // Recursively solve the rest
                    if solve_board(board) {
                        return true;
                    }

                    // Backtrack - remove the number and try next
                    board[index] = None;
                }
            }

            // No valid number found for this position
            false
        }
    }
}

/// Finds the index of the next empty cell in the board
///
/// # Arguments
/// * `board` - The board to search
///
/// # Returns
/// `Some(index)` if an empty cell is found, `None` if board is full
fn find_next_empty_cell(board: &[Option<u8>]) -> Option<usize> {
    board.iter().position(|&cell| cell.is_none())
}

/// Counts the number of empty cells in the board
///
/// # Arguments
/// * `board` - The board to analyze
///
/// # Returns
/// The number of empty cells
pub fn count_empty_cells(board: &[Option<u8>]) -> usize {
    board.iter().filter(|&&cell| cell.is_none()).count()
}

/// Counts the number of filled cells (clues) in the board
///
/// # Arguments
/// * `board` - The board to analyze
///
/// # Returns
/// The number of filled cells
pub fn count_clues(board: &[Option<u8>]) -> usize {
    board.iter().filter(|&&cell| cell.is_some()).count()
}

/// Checks if the board is completely filled
///
/// # Arguments
/// * `board` - The board to check
///
/// # Returns
/// `true` if all cells are filled
pub fn is_board_complete(board: &[Option<u8>]) -> bool {
    board.iter().all(|&cell| cell.is_some())
}

/// Checks if the board is valid (no rule violations) regardless of completion
///
/// # Arguments
/// * `board` - The board to validate
///
/// # Returns
/// `true` if no Sudoku rules are violated
pub fn is_board_valid(board: &[Option<u8>]) -> bool {
    validate_board(board).invalid_indices.is_empty()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_empty_board() {
        let board = vec![None; BOARD_SIZE];
        let result = validate_board(&board);
        assert!(result.invalid_indices.is_empty());
        assert!(!result.is_complete);
    }

    #[test]
    fn test_count_functions() {
        let mut board = vec![None; BOARD_SIZE];
        board[0] = Some(1);
        board[1] = Some(2);

        assert_eq!(count_empty_cells(&board), 79);
        assert_eq!(count_clues(&board), 2);
        assert!(!is_board_complete(&board));
    }

    #[test]
    fn test_find_next_empty_cell() {
        let mut board = vec![Some(1); BOARD_SIZE];
        assert_eq!(find_next_empty_cell(&board), None);

        board[5] = None;
        assert_eq!(find_next_empty_cell(&board), Some(5));
    }
}
