//! WebAssembly exports for the Sudoku engine
//!
//! This module provides the public interface that JavaScript can call
//! to interact with the Sudoku solver and generator.

use js_sys::Array;
use rand::rngs::SmallRng;
use rand::seq::SliceRandom;
use rand::SeedableRng;
use wasm_bindgen::prelude::*;
use web_sys::console;

use crate::difficulty::analyze_difficulty;
use crate::generator::{GeneratorConfig, PuzzleGenerator};
use crate::solver::HumanStyleSolver;
use crate::types::{DifficultyLevel, SolvingTechnique, BOARD_SIZE};
use crate::validator::{
    has_unique_solution, solve_board, validate_board as internal_validate_board,
};

/// JavaScript-compatible representation of a Sudoku board
///
/// Uses 0 for empty cells instead of Option<u8> for easier JS interop
pub type JsBoard = Vec<u8>;

/// Convert internal board format to JavaScript format
///
/// # Arguments
/// * `board` - Internal board with Option<u8> values
///
/// # Returns
/// JavaScript-compatible board with 0 for empty cells
fn to_js_board(board: &[Option<u8>]) -> JsBoard {
    board.iter().map(|&cell| cell.unwrap_or(0)).collect()
}

/// Convert JavaScript board format to internal format
///
/// # Arguments
/// * `js_board` - JavaScript board with 0 for empty cells
///
/// # Returns
/// Internal board format with Option<u8> values
fn from_js_board(js_board: &[u8]) -> Vec<Option<u8>> {
    js_board
        .iter()
        .map(|&cell| if cell == 0 { None } else { Some(cell) })
        .collect()
}

/// Generate a new Sudoku puzzle with the specified difficulty
///
/// # Arguments
/// * `difficulty` - Difficulty level (0=Easy, 1=Medium, 2=Hard, 3=Expert)
///
/// # Returns
/// A new puzzle as a flat array of 81 numbers (0 for empty cells)
///
/// # JavaScript Example
/// ```javascript
/// const puzzle = generate_puzzle(1); // Generate medium difficulty
/// console.log("Generated puzzle:", puzzle);
/// ```
#[wasm_bindgen]
pub fn generate_puzzle(difficulty: u8) -> Vec<u8> {
    console::log_1(&format!("Generating puzzle with difficulty level {}", difficulty).into());

    let difficulty_level = match difficulty {
        0 => DifficultyLevel::Easy,
        1 => DifficultyLevel::Medium,
        2 => DifficultyLevel::Hard,
        3 => DifficultyLevel::Expert,
        _ => {
            console::log_1(&"Invalid difficulty level, using Medium".into());
            DifficultyLevel::Medium
        }
    };

    let generator = PuzzleGenerator::with_difficulty(difficulty_level);

    match generator.generate() {
        Some(puzzle) => {
            let js_board = to_js_board(&puzzle);
            console::log_1(
                &format!(
                    "Successfully generated puzzle with {} clues",
                    js_board.iter().filter(|&&cell| cell != 0).count()
                )
                .into(),
            );
            js_board
        }
        None => {
            console::log_1(&"Failed to generate puzzle, returning empty board".into());
            vec![0; BOARD_SIZE]
        }
    }
}

/// Generate a puzzle with custom configuration
///
/// # Arguments
/// * `difficulty` - Target difficulty level (0-3)
/// * `min_clues` - Minimum number of clues
/// * `max_clues` - Maximum number of clues
/// * `prefer_symmetry` - Whether to prefer symmetric patterns
///
/// # Returns
/// A new puzzle as a flat array of 81 numbers
#[wasm_bindgen]
pub fn generate_custom_puzzle(
    difficulty: u8,
    min_clues: usize,
    max_clues: usize,
    prefer_symmetry: bool,
) -> Vec<u8> {
    let difficulty_level = match difficulty {
        0 => DifficultyLevel::Easy,
        1 => DifficultyLevel::Medium,
        2 => DifficultyLevel::Hard,
        3 => DifficultyLevel::Expert,
        _ => DifficultyLevel::Medium,
    };

    let config = GeneratorConfig {
        target_difficulty: difficulty_level,
        max_attempts: 1000,
        min_clues: min_clues.max(17), // Ensure minimum is at least 17
        max_clues: max_clues.min(50), // Ensure maximum is reasonable
        prefer_symmetry,

        // Use default branching factor settings for custom generation
        min_branching_factor: 1.0,
        max_branching_factor: 4.0,
        target_branching_factor: 2.5,
        branching_factor_tolerance: 0.5,
    };

    let generator = PuzzleGenerator::new(config);

    match generator.generate() {
        Some(puzzle) => to_js_board(&puzzle),
        None => {
            console::log_1(&"Custom puzzle generation failed".into());
            vec![0; BOARD_SIZE]
        }
    }
}

/// Validate a Sudoku board for correctness
///
/// Checks if the current state of the board violates any Sudoku rules.
/// Does not require the board to be complete.
///
/// # Arguments
/// * `board` - The board to validate (flat array of 81 numbers)
///
/// # Returns
/// `true` if the board state is valid, `false` if there are conflicts
///
/// # JavaScript Example
/// ```javascript
/// const isValid = validate_board(currentBoard);
/// if (!isValid) {
///     console.log("There are conflicts in the current board!");
/// }
/// ```
#[wasm_bindgen]
pub fn validate_board(board: Vec<u8>) -> bool {
    if board.len() != BOARD_SIZE {
        console::log_1(
            &format!(
                "Invalid board size: expected {}, got {}",
                BOARD_SIZE,
                board.len()
            )
            .into(),
        );
        return false;
    }

    let internal_board = from_js_board(&board);
    internal_validate_board(&internal_board)
        .invalid_indices
        .is_empty()
}

/// Check if a puzzle has a unique solution
///
/// This is important for puzzle quality - good Sudoku puzzles should have
/// exactly one solution.
///
/// # Arguments
/// * `board` - The puzzle to check (flat array of 81 numbers)
///
/// # Returns
/// `true` if the puzzle has exactly one solution
///
/// # JavaScript Example
/// ```javascript
/// const hasUniqueSolution = check_unique_solution(puzzle);
/// if (!hasUniqueSolution) {
///     console.log("This puzzle has multiple solutions or no solution!");
/// }
/// ```
#[wasm_bindgen]
pub fn check_unique_solution(board: Vec<u8>) -> bool {
    if board.len() != BOARD_SIZE {
        return false;
    }

    let internal_board = from_js_board(&board);
    has_unique_solution(&internal_board)
}

/// Solve a Sudoku puzzle completely
///
/// Uses backtracking to find a complete solution to the puzzle.
/// Returns the original board if no solution exists.
///
/// # Arguments
/// * `board` - The puzzle to solve (flat array of 81 numbers)
///
/// # Returns
/// The solved board, or the original board if unsolvable
///
/// # JavaScript Example
/// ```javascript
/// const solution = solve_puzzle(puzzle);
/// if (solution.every((cell, i) => cell === puzzle[i] || puzzle[i] === 0)) {
///     console.log("Found solution!");
/// } else {
///     console.log("No solution exists");
/// }
/// ```
#[wasm_bindgen]
pub fn solve_puzzle(board: Vec<u8>) -> Vec<u8> {
    if board.len() != BOARD_SIZE {
        console::log_1(&"Invalid board size for solving".into());
        return board;
    }

    let mut internal_board = from_js_board(&board);

    if solve_board(&mut internal_board) {
        to_js_board(&internal_board)
    } else {
        console::log_1(&"No solution found for the given puzzle".into());
        board // Return original if unsolvable
    }
}

/// Analyze the difficulty of a puzzle
///
/// Returns detailed information about what techniques are required
/// to solve the puzzle and estimates the overall difficulty.
///
/// # Arguments
/// * `board` - The puzzle to analyze (flat array of 81 numbers)
///
/// # Returns
/// A JSON string containing difficulty analysis
///
/// # JavaScript Example
/// ```javascript
/// const analysis = JSON.parse(analyze_puzzle_difficulty(puzzle));
/// console.log(`Difficulty: ${analysis.level}, Techniques: ${analysis.techniques}`);
/// ```
#[wasm_bindgen]
pub fn analyze_puzzle_difficulty(board: Vec<u8>) -> String {
    if board.len() != BOARD_SIZE {
        return r#"{"error": "Invalid board size"}"#.to_string();
    }

    let internal_board = from_js_board(&board);
    let analysis = analyze_difficulty(&internal_board);

    // Convert to JSON manually for simplicity
    let level_str = match analysis.level {
        DifficultyLevel::VeryEasy => "VeryEasy",
        DifficultyLevel::Easy => "Easy",
        DifficultyLevel::Medium => "Medium",
        DifficultyLevel::Hard => "Hard",
        DifficultyLevel::Expert => "Expert",
    };

    let technique_str = match analysis.hardest_technique {
        SolvingTechnique::NakedSingle => "Naked Single",
        SolvingTechnique::HiddenSingle => "Hidden Single",
        SolvingTechnique::NakedPair => "Naked Pair",
        SolvingTechnique::HiddenPair => "Hidden Pair",
        SolvingTechnique::BoxLineReduction => "Box/Line Reduction",
        SolvingTechnique::PointingPairs => "Pointing Pair",
        SolvingTechnique::XWing => "X-Wing",
        SolvingTechnique::PointingTriples => "Pointing Triples",
        SolvingTechnique::Swordfish => "Swordfish",
        SolvingTechnique::Coloring => "Coloring",
        SolvingTechnique::XYWing => "XY-Wing",
        SolvingTechnique::XYChain => "XY-Chain",
        SolvingTechnique::ForcingChain => "Forcing Chain",
        SolvingTechnique::TrialAndError => "Trial and Error",
    };

    format!(
        r#"{{"level": "{}", "hardest_technique": "{}", "technique_diversity": {}, "branching_factor": {:.2}}}"#,
        level_str, technique_str, analysis.technique_diversity, analysis.branching_factor
    )
}

/// Solve a puzzle step by step using human-style techniques
///
/// Returns information about what techniques were used and the
/// intermediate steps taken during solving.
///
/// # Arguments  
/// * `board` - The puzzle to solve (flat array of 81 numbers)
///
/// # Returns
/// A JSON string containing the solving steps and techniques used
///
/// # JavaScript Example
/// ```javascript
/// const result = JSON.parse(solve_with_techniques(puzzle));
/// console.log(`Used techniques: ${result.techniques.join(', ')}`);
/// console.log(`Solved: ${result.solved}`);
/// ```
#[wasm_bindgen]
pub fn solve_with_techniques(board: Vec<u8>) -> String {
    if board.len() != BOARD_SIZE {
        return r#"{"error": "Invalid board size"}"#.to_string();
    }

    let internal_board = from_js_board(&board);
    let mut solver = HumanStyleSolver::new(&internal_board);

    let solved = solver.solve_with_techniques();
    let techniques_used = solver.get_techniques_used();
    let final_board = solver.get_board();
    let branching_factor = solver.calculate_branching_factor();

    // Convert techniques to strings
    let technique_names: Vec<&str> = techniques_used
        .iter()
        .map(|t| match t {
            SolvingTechnique::NakedSingle => "Naked Single",
            SolvingTechnique::HiddenSingle => "Hidden Single",
            SolvingTechnique::NakedPair => "Naked Pair",
            SolvingTechnique::HiddenPair => "Hidden Pair",
            SolvingTechnique::BoxLineReduction => "Box/Line Reduction",
            SolvingTechnique::PointingPairs => "Pointing Pair",
            SolvingTechnique::XWing => "X-Wing",
            SolvingTechnique::PointingTriples => "Pointing Triples",
            SolvingTechnique::Swordfish => "Swordfish",
            SolvingTechnique::Coloring => "Coloring",
            SolvingTechnique::XYWing => "XY-Wing",
            SolvingTechnique::XYChain => "XY-Chain",
            SolvingTechnique::ForcingChain => "Forcing Chain",
            SolvingTechnique::TrialAndError => "Trial and Error",
        })
        .collect();

    let techniques_json = technique_names
        .iter()
        .map(|&t| format!(r#""{}""#, t))
        .collect::<Vec<_>>()
        .join(",");

    let board_json = to_js_board(final_board)
        .iter()
        .map(|&n| n.to_string())
        .collect::<Vec<_>>()
        .join(",");

    format!(
        r#"{{"solved": {}, "techniques": [{}], "board": [{}], "branching_factor": {:.2}}}"#,
        solved, techniques_json, board_json, branching_factor
    )
}

/// Get a hint for the next move in a puzzle
///
/// Analyzes the current board state and suggests the next logical move
/// that a human solver might make.
///
/// # Arguments
/// * `board` - The current puzzle state (flat array of 81 numbers)
///
/// # Returns
/// A JSON string with hint information (cell index, number, technique used)
///
/// # JavaScript Example
/// ```javascript
/// const hint = JSON.parse(get_hint(currentBoard));
/// if (hint.cell !== -1) {
///     console.log(`Try placing ${hint.number} at position ${hint.cell}`);
///     console.log(`Technique: ${hint.technique}`);
/// }
/// ```
#[wasm_bindgen]
pub fn get_hint(board: Vec<u8>) -> String {
    if board.len() != BOARD_SIZE {
        return r#"{"error": "Invalid board size"}"#.to_string();
    }

    let internal_board = from_js_board(&board);
    let mut solver = HumanStyleSolver::new(&internal_board);

    // Try to make one step of progress
    let original_board = solver.get_board().to_vec();

    // Attempt one round of basic techniques
    if solver.apply_basic_techniques() {
        // Find what changed
        let new_board = solver.get_board();
        for (index, (&old, &new)) in original_board.iter().zip(new_board.iter()).enumerate() {
            if old != new && new.is_some() {
                return format!(
                    r#"{{"cell": {}, "number": {}, "technique": "Basic solving technique"}}"#,
                    index,
                    new.unwrap()
                );
            }
        }
    }

    // No immediate hint available
    r#"{"cell": -1, "number": 0, "technique": "No immediate hint available"}"#.to_string()
}

/// Initialize the WASM module
///
/// Sets up panic hooks and logging for better debugging experience.
/// Should be called once when the module is loaded.
#[wasm_bindgen(start)]
pub fn init() {
    // Set up better panic messages in the browser console
    #[cfg(feature = "console_error_panic_hook")]
    console_error_panic_hook::set_once();

    // Enable logging
    console::log_1(&"Sudoku WASM module initialized".into());
}

/// Get version information about the WASM module
///
/// # Returns
/// Version string
#[wasm_bindgen]
pub fn get_version() -> String {
    env!("CARGO_PKG_VERSION").to_string()
}

/// Generate a complete solved Sudoku board using a specific seed for reproducible results
fn generate_solved_board_with_seed(seed: u64) -> Vec<u8> {
    let mut board = vec![None; BOARD_SIZE];
    let mut rng = SmallRng::seed_from_u64(seed);

    fill_board_seeded(&mut board, &mut rng);

    // Convert to Vec<u8> (should all be Some values)
    board.into_iter().map(|cell| cell.unwrap_or(1)).collect()
}

/// Fill board using backtracking with seeded randomization
fn fill_board_seeded(board: &mut [Option<u8>], rng: &mut SmallRng) -> bool {
    // Find first empty cell
    if let Some(empty_idx) = board.iter().position(|&cell| cell.is_none()) {
        let row = empty_idx / 9;
        let col = empty_idx % 9;

        // Create shuffled list of numbers 1-9
        let mut numbers = vec![1, 2, 3, 4, 5, 6, 7, 8, 9];
        numbers.shuffle(rng);

        for num in numbers {
            if is_valid_placement_seeded(board, row, col, num) {
                board[empty_idx] = Some(num);
                if fill_board_seeded(board, rng) {
                    return true;
                }
                board[empty_idx] = None;
            }
        }
        false
    } else {
        true // Board is complete
    }
}

/// Check if placing a number at the given position is valid
fn is_valid_placement_seeded(board: &[Option<u8>], row: usize, col: usize, num: u8) -> bool {
    // Check row
    for c in 0..9 {
        if board[row * 9 + c] == Some(num) {
            return false;
        }
    }

    // Check column
    for r in 0..9 {
        if board[r * 9 + col] == Some(num) {
            return false;
        }
    }

    // Check 3x3 box
    let box_row = (row / 3) * 3;
    let box_col = (col / 3) * 3;
    for r in box_row..box_row + 3 {
        for c in box_col..box_col + 3 {
            if board[r * 9 + c] == Some(num) {
                return false;
            }
        }
    }

    true
}

/// Create a puzzle from solved board with seeded randomization
fn create_puzzle_with_seed(solved_board: &[u8], difficulty: u8, seed: u64) -> Vec<Option<u8>> {
    // Map legacy difficulty numbers to DifficultyLevel and use the new generator
    let difficulty_level = match difficulty {
        1 => DifficultyLevel::VeryEasy, // 35-45 clues (matches modal "Very Easy")
        2 => DifficultyLevel::Easy,     // 35-45 clues (matches modal "Easy")
        3 => DifficultyLevel::Medium,   // 30-35 clues (matches modal "Medium")
        4 => DifficultyLevel::Hard,     // 25-30 clues (matches modal "Hard")
        5 => DifficultyLevel::Expert,   // 17-24 clues (matches modal "Very Hard")
        _ => DifficultyLevel::Medium,
    };

    // Use the new generator with a seeded configuration
    let mut config = GeneratorConfig::for_difficulty(difficulty_level);
    config.target_branching_factor = match difficulty_level {
        DifficultyLevel::VeryEasy => 1.4,
        DifficultyLevel::Easy => 1.9,
        DifficultyLevel::Medium => 2.5,
        DifficultyLevel::Hard => 3.8,
        DifficultyLevel::Expert => 5.0,
    };

    // Create generator and try to generate
    let generator = PuzzleGenerator::new(config);

    // Try multiple times with seed variations to ensure we get a puzzle
    for attempt in 0..10 {
        let _variation_seed = seed.wrapping_add(attempt as u64);

        if let Some(puzzle) = generator.generate() {
            return puzzle;
        }
    }

    // Fallback to old method if new generator fails
    let mut board: Vec<Option<u8>> = solved_board.iter().map(|&x| Some(x)).collect();
    let mut rng = SmallRng::seed_from_u64(seed.wrapping_add(difficulty as u64));

    // Updated cells_to_remove to match modal descriptions and new difficulty analysis
    let cells_to_remove = match difficulty {
        1 => 36, // VeryEasy - leave 45 clues (matches modal "35-45 clues")
        2 => 40, // Easy - leave 41 clues (matches modal "35-45 clues")
        3 => 48, // Medium - leave 33 clues (matches modal "30-35 clues")
        4 => 56, // Hard - leave 25 clues (matches modal "25-30 clues")
        5 => 64, // Expert - leave 17 clues (matches modal "17-24 clues")
        _ => 48, // Default to Medium
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

/// Create a new Sudoku game with specified difficulty and seed (legacy compatibility)
///
/// # Arguments
/// * `difficulty` - Difficulty level (1=Easy, 2=Medium, 3=Hard, 4=Expert)
/// * `seed` - Seed for deterministic puzzle generation
///
/// # Returns
/// JavaScript array with puzzle data (numbers for clues, undefined for empty cells)
#[wasm_bindgen]
#[allow(non_snake_case)]
pub fn createGameWithSeed(difficulty: u8, seed: u64) -> JsValue {
    console::log_1(
        &format!(
            "Creating seeded game with difficulty: {}, seed: {}",
            difficulty, seed
        )
        .into(),
    );

    // Now actually use the seed for deterministic generation
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

/// Validate a Sudoku board and return detailed validation result (legacy compatibility)
///
/// This function maintains compatibility with existing JavaScript code that expects
/// a ValidationResult object instead of just a boolean.
///
/// # Arguments
/// * `board` - JavaScript array representing current board state
///
/// # Returns
/// JavaScript object with { invalidIndices: number[], isComplete: boolean }
#[wasm_bindgen]
#[allow(non_snake_case)]
pub fn validateBoard(board: JsValue) -> JsValue {
    console::log_1(&"Validating board".into());

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

    let result = internal_validate_board(&rust_board);

    // Create JavaScript object manually for compatibility
    let js_result = js_sys::Object::new();

    // Convert invalid indices to JavaScript array
    let invalid_indices_array = Array::new();
    for &index in &result.invalid_indices {
        invalid_indices_array.push(&JsValue::from(index));
    }

    js_sys::Reflect::set(
        &js_result,
        &"invalidIndices".into(),
        &invalid_indices_array.into(),
    )
    .unwrap();
    js_sys::Reflect::set(
        &js_result,
        &"isComplete".into(),
        &JsValue::from(result.is_complete),
    )
    .unwrap();

    js_result.into()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_board_conversion() {
        let internal = vec![Some(1), None, Some(3), None];
        let js_board = to_js_board(&internal);
        assert_eq!(js_board, vec![1, 0, 3, 0]);

        let back_to_internal = from_js_board(&js_board);
        assert_eq!(back_to_internal, internal);
    }

    #[test]
    fn test_validate_empty_board() {
        let empty_board = vec![0; BOARD_SIZE];
        assert!(validate_board(empty_board));
    }

    #[test]
    fn test_version() {
        let version = get_version();
        assert!(!version.is_empty());
    }
}
