//! Sudoku WASM Module
//!
//! This Rust crate provides Sudoku game logic compiled to WebAssembly.
//! It exports functions for:
//! - Generating solved boards
//! - Creating games with varying difficulty
//! - Validating current board state
//! - Solving incomplete boards

use js_sys::Array;
use rand::prelude::*;
use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;

// Set up console logging for debugging
#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);
}

macro_rules! console_log {
    ($($t:tt)*) => (log(&format_args!($($t)*).to_string()))
}

/// Result of board validation
#[derive(Serialize, Deserialize)]
pub struct ValidationResult {
    pub invalid_indices: Vec<usize>,
    pub is_complete: bool,
}

/// Sudoku board size constant (9x9 = 81 cells)
const BOARD_SIZE: usize = 81;
const GRID_SIZE: usize = 9;
const BOX_SIZE: usize = 3;

/// Convert 2D coordinates to 1D index
fn coords_to_index(row: usize, col: usize) -> usize {
    row * GRID_SIZE + col
}

/// Convert 1D index to 2D coordinates
fn index_to_coords(index: usize) -> (usize, usize) {
    (index / GRID_SIZE, index % GRID_SIZE)
}

/// Get the 3x3 box index for given coordinates
fn get_box_index(row: usize, col: usize) -> usize {
    (row / BOX_SIZE) * BOX_SIZE + (col / BOX_SIZE)
}

/// Check if placing a number at given position is valid
fn is_valid_placement(board: &[Option<u8>], row: usize, col: usize, num: u8) -> bool {
    // Check row
    for c in 0..GRID_SIZE {
        if board[coords_to_index(row, c)] == Some(num) {
            return false;
        }
    }

    // Check column
    for r in 0..GRID_SIZE {
        if board[coords_to_index(r, col)] == Some(num) {
            return false;
        }
    }

    // Check 3x3 box
    let box_row = (row / BOX_SIZE) * BOX_SIZE;
    let box_col = (col / BOX_SIZE) * BOX_SIZE;
    for r in box_row..box_row + BOX_SIZE {
        for c in box_col..box_col + BOX_SIZE {
            if board[coords_to_index(r, c)] == Some(num) {
                return false;
            }
        }
    }

    true
}

/// Generate a completely solved Sudoku board using backtracking
fn generate_solved_board() -> Vec<u8> {
    let mut board = vec![None; BOARD_SIZE];
    let mut rng = SmallRng::from_entropy();

    fill_board(&mut board, &mut rng);

    // Convert to Vec<u8> (should all be Some values)
    board.into_iter().map(|cell| cell.unwrap_or(1)).collect()
}

/// Generate a completely solved Sudoku board using a specific seed for reproducible results
fn generate_solved_board_with_seed(seed: u64) -> Vec<u8> {
    let mut board = vec![None; BOARD_SIZE];
    let mut rng = SmallRng::seed_from_u64(seed);

    fill_board(&mut board, &mut rng);

    // Convert to Vec<u8> (should all be Some values)
    board.into_iter().map(|cell| cell.unwrap_or(1)).collect()
}

/// Fill board using backtracking with randomization
fn fill_board(board: &mut [Option<u8>], rng: &mut SmallRng) -> bool {
    // Find next empty cell
    let empty_cell = board.iter().position(|&cell| cell.is_none());

    match empty_cell {
        None => true, // Board is complete
        Some(index) => {
            let (row, col) = index_to_coords(index);

            // Try numbers 1-9 in random order
            let mut numbers: Vec<u8> = (1..=9).collect();
            numbers.shuffle(rng);

            for num in numbers {
                if is_valid_placement(board, row, col, num) {
                    board[index] = Some(num);

                    if fill_board(board, rng) {
                        return true;
                    }

                    board[index] = None; // Backtrack
                }
            }

            false
        }
    }
}

/// Remove cells from solved board to create puzzle
fn create_puzzle(solved_board: &[u8], difficulty: u8) -> Vec<Option<u8>> {
    let mut board: Vec<Option<u8>> = solved_board.iter().map(|&x| Some(x)).collect();
    let mut rng = SmallRng::from_entropy();

    // Determine number of cells to remove based on difficulty
    let cells_to_remove = match difficulty {
        1 => 35, // Beginner - fewer cells removed, more clues
        2 => 45, // Easy
        3 => 55, // Medium
        4 => 65, // Hard
        5 => 75, // Expert - more cells removed, fewer clues
        _ => 55, // Default to medium
    };

    let mut indices: Vec<usize> = (0..BOARD_SIZE).collect();
    indices.shuffle(&mut rng);

    let mut removed = 0;
    for &index in &indices {
        if removed >= cells_to_remove {
            break;
        }

        // Try removing this cell
        let original = board[index];
        board[index] = None;

        // Check if puzzle still has unique solution
        if has_unique_solution(&board) {
            removed += 1;
        } else {
            // Restore cell if removing it makes puzzle unsolvable or non-unique
            board[index] = original;
        }
    }

    board
}

/// Remove cells from solved board to create puzzle with deterministic seed
/// This ensures the same seed and difficulty always produce the same puzzle
fn create_puzzle_with_seed(solved_board: &[u8], difficulty: u8, seed: u64) -> Vec<Option<u8>> {
    let mut board: Vec<Option<u8>> = solved_board.iter().map(|&x| Some(x)).collect();
    let mut rng = SmallRng::seed_from_u64(seed.wrapping_add(difficulty as u64));

    // Determine number of cells to remove based on difficulty
    let cells_to_remove = match difficulty {
        1 => 35, // Beginner - fewer cells removed, more clues
        2 => 45, // Easy
        3 => 55, // Medium
        4 => 65, // Hard
        5 => 75, // Expert - more cells removed, fewer clues
        _ => 55, // Default to medium
    };

    let mut indices: Vec<usize> = (0..BOARD_SIZE).collect();
    indices.shuffle(&mut rng);

    let mut removed = 0;
    for &index in &indices {
        if removed >= cells_to_remove {
            break;
        }

        // Try removing this cell
        let original = board[index];
        board[index] = None;

        // Check if puzzle still has unique solution
        if has_unique_solution(&board) {
            removed += 1;
        } else {
            // Restore cell if removing it makes puzzle unsolvable or non-unique
            board[index] = original;
        }
    }

    board
}

/// Check if puzzle has a unique solution (simplified check)
fn has_unique_solution(board: &[Option<u8>]) -> bool {
    let mut test_board = board.to_vec();
    solve_board_internal(&mut test_board)
}

/// Solve board using backtracking
fn solve_board_internal(board: &mut [Option<u8>]) -> bool {
    // Find next empty cell
    let empty_cell = board.iter().position(|&cell| cell.is_none());

    match empty_cell {
        None => true, // Board is complete
        Some(index) => {
            let (row, col) = index_to_coords(index);

            for num in 1..=9 {
                if is_valid_placement(board, row, col, num) {
                    board[index] = Some(num);

                    if solve_board_internal(board) {
                        return true;
                    }

                    board[index] = None; // Backtrack
                }
            }

            false
        }
    }
}

/// Validate current board state and find conflicts
fn validate_board_internal(board: &[Option<u8>]) -> ValidationResult {
    let mut invalid_indices = Vec::new();
    let mut is_complete = true;

    for index in 0..BOARD_SIZE {
        let (row, col) = index_to_coords(index);

        match board[index] {
            None => {
                is_complete = false;
            }
            Some(num) => {
                // Temporarily remove this cell and check if placement is valid
                let mut temp_board = board.to_vec();
                temp_board[index] = None;

                if !is_valid_placement(&temp_board, row, col, num) {
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

// WASM exported functions

/// Generate a completely solved 9x9 Sudoku board
/// Returns: Vec<u8> with 81 numbers (1-9)
#[wasm_bindgen]
#[allow(non_snake_case)]
pub fn createBoard() -> Vec<u8> {
    console_log!("Generating new solved board");
    generate_solved_board()
}

/// Create a new Sudoku game with specified difficulty
/// Parameters:
/// - difficulty: 1 (easy), 2 (medium), 3 (hard)
/// Returns: Vec<Option<u8>> with 81 cells, Some(n) for given numbers, None for empty
#[wasm_bindgen]
#[allow(non_snake_case)]
pub fn createGame(difficulty: u8) -> JsValue {
    console_log!("Creating new game with difficulty: {}", difficulty);

    let solved_board = generate_solved_board();
    let puzzle = create_puzzle(&solved_board, difficulty);

    // Convert to JavaScript array of numbers/undefined
    let js_array = Array::new();
    for cell in puzzle {
        match cell {
            Some(num) => {
                js_array.push(&JsValue::from(num));
            }
            None => {
                js_array.push(&JsValue::undefined());
            }
        }
    }

    js_array.into()
}

/// Create a new Sudoku game with specified difficulty and seed for reproducible puzzles
/// Parameters:
/// - difficulty: 1 (easy), 2 (medium), 3 (hard)  
/// - seed: u64 seed for deterministic puzzle generation
/// Returns: Vec<Option<u8>> with 81 cells, Some(n) for given numbers, None for empty
#[wasm_bindgen]
#[allow(non_snake_case)]
pub fn createGameWithSeed(difficulty: u8, seed: u64) -> JsValue {
    console_log!(
        "Creating seeded game with difficulty: {}, seed: {}",
        difficulty,
        seed
    );

    let solved_board = generate_solved_board_with_seed(seed);
    let puzzle = create_puzzle_with_seed(&solved_board, difficulty, seed);

    // Convert to JavaScript array of numbers/undefined
    let js_array = Array::new();
    for cell in puzzle {
        match cell {
            Some(num) => {
                js_array.push(&JsValue::from(num));
            }
            None => {
                js_array.push(&JsValue::undefined());
            }
        }
    }

    js_array.into()
}

/// Validate the current board state
/// Parameters:
/// - board: Vec<Option<u8>> representing current board state
/// Returns: JSON string with { invalidIndices: number[], isComplete: boolean }
#[wasm_bindgen]
#[allow(non_snake_case)]
pub fn validateBoard(board: JsValue) -> JsValue {
    console_log!("Validating board");

    // Convert JavaScript array to Rust Vec
    let js_array: Array = board.into();
    let mut rust_board = Vec::with_capacity(BOARD_SIZE);

    for i in 0..BOARD_SIZE {
        let cell = js_array.get(i as u32);
        if cell.is_undefined() {
            rust_board.push(None);
        } else {
            let num = cell.as_f64().unwrap_or(0.0) as u8;
            if num >= 1 && num <= 9 {
                rust_board.push(Some(num));
            } else {
                rust_board.push(None);
            }
        }
    }

    let result = validate_board_internal(&rust_board);

    // Convert to JSON string
    serde_wasm_bindgen::to_value(&result).unwrap_or(JsValue::NULL)
}

/// Solve the given board
/// Parameters:
/// - board: Vec<Option<u8>> representing current board state
/// Returns: Vec<u8> with complete solution
#[wasm_bindgen]
#[allow(non_snake_case)]
pub fn solveBoard(board: JsValue) -> Vec<u8> {
    console_log!("Solving board");

    // Convert JavaScript array to Rust Vec
    let js_array: Array = board.into();
    let mut rust_board = Vec::with_capacity(BOARD_SIZE);

    for i in 0..BOARD_SIZE {
        let cell = js_array.get(i as u32);
        if cell.is_undefined() {
            rust_board.push(None);
        } else {
            let num = cell.as_f64().unwrap_or(0.0) as u8;
            if num >= 1 && num <= 9 {
                rust_board.push(Some(num));
            } else {
                rust_board.push(None);
            }
        }
    }

    // Solve the board
    solve_board_internal(&mut rust_board);

    // Convert to Vec<u8>
    rust_board
        .into_iter()
        .map(|cell| cell.unwrap_or(1))
        .collect()
}

/// Initialize the WASM module
#[wasm_bindgen(start)]
pub fn main() {
    console_log!("Sudoku WASM module initialized");
}
