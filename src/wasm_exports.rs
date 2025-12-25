//! WebAssembly exports for the Sudoku engine
//!
//! This module provides the public interface that JavaScript can call
//! to interact with the Sudoku solver and generator.

use js_sys::{Array, Function};
use rand::rngs::SmallRng;
use rand::seq::SliceRandom;
use rand::SeedableRng;
use wasm_bindgen::prelude::*;
use web_sys::console;

use crate::difficulty::analyze_difficulty;
use crate::generator::{
    report_progress, report_progress_with_meta, reset_progress_tracker, search_progress,
    GeneratorConfig, ProgressMeta, PuzzleGenerator,
};
use crate::solver::HumanStyleSolver;
use crate::types::{DifficultyAnalysis, DifficultyLevel, SolvingTechnique, BOARD_SIZE};
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

/// Register a JS callback to receive generation progress updates
///
/// Callback signature: (progress: number, stage: string) => void
#[wasm_bindgen]
pub fn register_progress_callback(callback: Function) {
    crate::generator::set_generation_progress_callback(Some(callback));
}

/// Clear the registered generation progress callback
#[wasm_bindgen]
pub fn clear_progress_callback() {
    crate::generator::set_generation_progress_callback(None);
}

/// Generate a new Sudoku puzzle with the specified difficulty
///
/// # Arguments
/// * `difficulty` - Difficulty level (1=VeryEasy, 2=Easy, 3=Medium, 4=Hard, 5=Expert)
///
/// # Returns
/// A new puzzle as a flat array of 81 numbers (0 for empty cells)
///
/// # JavaScript Example
/// ```javascript
/// const puzzle = generate_puzzle(3); // Generate medium difficulty
/// console.log("Generated puzzle:", puzzle);
/// ```
#[wasm_bindgen]
pub fn generate_puzzle(difficulty: u8) -> Vec<u8> {
    console::log_1(&format!("Generating puzzle with difficulty level {}", difficulty).into());

    let difficulty_level = match difficulty {
        1 => DifficultyLevel::VeryEasy,
        2 => DifficultyLevel::Easy,
        3 => DifficultyLevel::Medium,
        4 => DifficultyLevel::Hard,
        5 => DifficultyLevel::Expert,
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
/// * `difficulty` - Target difficulty level (1-5)
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
        1 => DifficultyLevel::VeryEasy,
        2 => DifficultyLevel::Easy,
        3 => DifficultyLevel::Medium,
        4 => DifficultyLevel::Hard,
        5 => DifficultyLevel::Expert,
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
        unique_check_interval: 4,
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
/// exactly one solution. Uses a solution-counting backtracking routine.
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
    let difficulty_level = match difficulty {
        1 => DifficultyLevel::VeryEasy,
        2 => DifficultyLevel::Easy,
        3 => DifficultyLevel::Medium,
        4 => DifficultyLevel::Hard,
        5 => DifficultyLevel::Expert,
        _ => DifficultyLevel::Medium,
    };
    let config = GeneratorConfig::for_difficulty(difficulty_level);
    let generator = PuzzleGenerator::new(config.clone());
    let base_seed = seed.wrapping_add(difficulty as u64);

    reset_progress_tracker();
    report_progress_with_meta(
        0.05,
        "Preparing generator",
        Some(ProgressMeta {
            attempt: Some(0),
            max_attempts: Some(config.max_attempts),
            best_score: None,
            best_clue_count: None,
        }),
    );

    let mut best_puzzle: Option<Vec<Option<u8>>> = None;
    let mut best_score = f64::INFINITY;
    let mut best_clue_count: Option<usize> = None;
    let removal_target = (BOARD_SIZE - config.min_clues).max(1);

    for attempt in 0..config.max_attempts {
        let mut board: Vec<Option<u8>> = solved_board.iter().map(|&x| Some(x)).collect();
        let mut attempt_rng = SmallRng::seed_from_u64(base_seed.wrapping_add(attempt as u64));
        let order = seeded_removal_order(&mut attempt_rng, config.prefer_symmetry);
        let mut since_unique_check = 0;
        if attempt % 10 == 0 {
            let pct = 0.05 + (attempt as f64 / config.max_attempts as f64) * 0.15;
            report_progress(pct.min(0.2), "Searching puzzle variants");
        }

        if attempt % 5 == 0 {
            report_progress_with_meta(
                search_progress(attempt, config.max_attempts),
                "Evaluating candidate puzzles",
                Some(ProgressMeta {
                    attempt: Some(attempt),
                    max_attempts: Some(config.max_attempts),
                    best_score: None,
                    best_clue_count: None,
                }),
            );
        }

        for (step, idx) in order.into_iter().enumerate() {
            let saved = board[idx];
            board[idx] = None;

            let clue_count = board.iter().filter(|c| c.is_some()).count();
            if clue_count < config.min_clues {
                board[idx] = saved;
                break;
            }

            if step % 3 == 0 {
                let removed = BOARD_SIZE.saturating_sub(clue_count);
                let removal_fraction = (removed as f64 / removal_target as f64).min(1.0);
                let pct = 0.2 + removal_fraction * 0.4; // cap carving around 0.6
                report_progress(pct.min(0.6), "Carving clues");
            }

            let needs_unique_check = since_unique_check >= config.unique_check_interval
                || clue_count <= config.min_clues + 2;
            if needs_unique_check && !has_unique_solution(&board) {
                board[idx] = saved;
                since_unique_check = 0;
                continue;
            }

            since_unique_check = if needs_unique_check {
                0
            } else {
                since_unique_check + 1
            };
        }

        if !has_unique_solution(&board) {
            continue;
        }

        let analysis = analyze_difficulty(&board);
        let branching_factor = generator.calculate_branching_factor(&board);

        if seeded_meets_constraints(&config, &board, &analysis, branching_factor) {
            report_progress_with_meta(
                0.8,
                "Validating uniqueness",
                Some(ProgressMeta {
                    attempt: Some(attempt),
                    max_attempts: Some(config.max_attempts),
                    best_score: Some(best_score),
                    best_clue_count,
                }),
            );
            report_progress_with_meta(
                0.9,
                "Analyzing difficulty",
                Some(ProgressMeta {
                    attempt: Some(attempt),
                    max_attempts: Some(config.max_attempts),
                    best_score: Some(best_score),
                    best_clue_count,
                }),
            );
            report_progress_with_meta(
                0.98,
                "Finalizing puzzle",
                Some(ProgressMeta {
                    attempt: Some(attempt),
                    max_attempts: Some(config.max_attempts),
                    best_score: Some(best_score),
                    best_clue_count,
                }),
            );
            return board;
        }

        let clue_count = board.iter().filter(|c| c.is_some()).count();
        let bf_diff = (branching_factor - config.target_branching_factor).abs();
        let score = bf_diff + (clue_count as f64 - config.min_clues as f64).abs() * 0.1;
        if score < best_score {
            best_score = score;
            best_puzzle = Some(board);
            best_clue_count = Some(clue_count);
            report_progress_with_meta(
                search_progress(attempt, config.max_attempts),
                "Evaluating candidate puzzles",
                Some(ProgressMeta {
                    attempt: Some(attempt),
                    max_attempts: Some(config.max_attempts),
                    best_score: Some(best_score),
                    best_clue_count,
                }),
            );
        }

        // Surface progress for long-running search across attempts
        if attempt % 25 == 0 {
            report_progress_with_meta(
                search_progress(attempt, config.max_attempts),
                "Evaluating candidate puzzles",
                Some(ProgressMeta {
                    attempt: Some(attempt),
                    max_attempts: Some(config.max_attempts),
                    best_score: best_score.into(),
                    best_clue_count,
                }),
            );
        }
    }

    let mut puzzle = best_puzzle.unwrap_or_else(|| solved_board.iter().map(|&x| Some(x)).collect());

    // Safety net: ensure we don't return a fully filled board if constraints couldn't be met
    enforce_clue_bounds_with_seed(
        &mut puzzle,
        &config,
        base_seed
            .wrapping_mul(0x9E37_79B9_7F4A_7C15)
            .wrapping_add(0xA5A5_5A5A),
    );

    report_progress_with_meta(
        0.98,
        "Finalizing puzzle",
        Some(ProgressMeta {
            attempt: Some(config.max_attempts),
            max_attempts: Some(config.max_attempts),
            best_score: Some(best_score),
            best_clue_count,
        }),
    );
    puzzle
}

fn enforce_clue_bounds_with_seed(
    puzzle: &mut Vec<Option<u8>>,
    config: &GeneratorConfig,
    seed: u64,
) {
    let mut clue_count = puzzle.iter().filter(|c| c.is_some()).count();

    // If we're already within bounds and have blanks, leave the puzzle as-is
    if clue_count < BOARD_SIZE && clue_count <= config.max_clues {
        return;
    }

    let mut rng = SmallRng::seed_from_u64(seed);
    let mut removal_order = seeded_removal_order(&mut rng, config.prefer_symmetry);
    let mut since_unique_check = 0usize;

    for idx in removal_order.drain(..) {
        if clue_count <= config.max_clues {
            break;
        }
        if puzzle[idx].is_none() {
            continue;
        }

        let saved = puzzle[idx];
        puzzle[idx] = None;
        clue_count -= 1;

        let needs_unique_check = since_unique_check >= config.unique_check_interval
            || clue_count <= config.min_clues + 2;

        let mut revert = clue_count < config.min_clues;
        if !revert && needs_unique_check && !has_unique_solution(puzzle) {
            revert = true;
        }

        if revert {
            puzzle[idx] = saved;
            clue_count += 1;
            if needs_unique_check {
                since_unique_check = 0;
            }
        } else {
            since_unique_check = if needs_unique_check {
                0
            } else {
                since_unique_check + 1
            };
        }
    }

    // If we still have too many clues, do a final deterministic trim pass
    if clue_count > config.max_clues {
        let mut indices: Vec<usize> = (0..BOARD_SIZE).collect();
        indices.shuffle(&mut rng);

        for idx in indices {
            if clue_count <= config.max_clues {
                break;
            }
            if puzzle[idx].is_none() {
                continue;
            }

            let saved = puzzle[idx];
            puzzle[idx] = None;
            clue_count -= 1;

            if clue_count < config.min_clues || !has_unique_solution(puzzle) {
                puzzle[idx] = saved;
                clue_count += 1;
            }
        }
    }
}

fn seeded_meets_constraints(
    config: &GeneratorConfig,
    puzzle: &[Option<u8>],
    analysis: &DifficultyAnalysis,
    branching_factor: f64,
) -> bool {
    let clue_count = puzzle.iter().filter(|c| c.is_some()).count();

    if clue_count < config.min_clues || clue_count > config.max_clues {
        return false;
    }

    if !seeded_difficulty_matches_target(config.target_difficulty, analysis) {
        return false;
    }

    if branching_factor < config.min_branching_factor
        || branching_factor > config.max_branching_factor
    {
        return false;
    }

    let bf_diff = (branching_factor - config.target_branching_factor).abs();
    bf_diff <= config.branching_factor_tolerance
}

fn seeded_difficulty_matches_target(
    target: DifficultyLevel,
    analysis: &DifficultyAnalysis,
) -> bool {
    use SolvingTechnique::*;
    match target {
        DifficultyLevel::VeryEasy => analysis.hardest_technique <= HiddenSingle,
        DifficultyLevel::Easy => analysis.hardest_technique <= HiddenSingle,
        DifficultyLevel::Medium => analysis.hardest_technique <= BoxLineReduction,
        DifficultyLevel::Hard => {
            analysis.hardest_technique >= XWing && analysis.hardest_technique <= Swordfish
        }
        DifficultyLevel::Expert => analysis.hardest_technique >= XYWing,
    }
}

fn seeded_removal_order(rng: &mut SmallRng, prefer_symmetry: bool) -> Vec<usize> {
    let mut indices: Vec<usize> = (0..BOARD_SIZE).collect();

    if prefer_symmetry {
        let mut pairs = Vec::<(usize, usize)>::new();
        let mut seen = vec![false; BOARD_SIZE];
        for i in 0..BOARD_SIZE {
            if seen[i] {
                continue;
            }
            let s = seeded_symmetric_index(i);
            if s != i && !seen[s] {
                pairs.push((i, s));
                seen[i] = true;
                seen[s] = true;
            } else {
                pairs.push((i, i));
                seen[i] = true;
            }
        }
        pairs.shuffle(rng);
        indices = pairs
            .into_iter()
            .flat_map(|(a, b)| if a == b { vec![a] } else { vec![a, b] })
            .collect();
    } else {
        indices.shuffle(rng);
    }

    indices
}

fn seeded_symmetric_index(index: usize) -> usize {
    let row = index / 9;
    let col = index % 9;
    (8 - row) * 9 + (8 - col)
}

/// Create a new Sudoku game with specified difficulty and seed (legacy compatibility)
///
/// # Arguments
/// * `difficulty` - Difficulty level (1=VeryEasy, 2=Easy, 3=Medium, 4=Hard, 5=Expert)
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
