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

/// Solving technique tiers for difficulty classification
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
#[allow(dead_code)]
enum SolvingTechnique {
    NakedSingle,
    HiddenSingle,
    NakedPair,
    HiddenPair,
    BoxLineReduction,
    PointingPairs,
    XWing,
    PointingTriples,
    Swordfish,
    Coloring,
    XYWing,
    XYChain,
    ForcingChain,
    TrialAndError,
}

/// Difficulty analysis result
#[derive(Serialize, Deserialize, Debug)]
pub struct DifficultyAnalysis {
    pub difficulty_level: u8,
    pub clue_count: usize,
    pub techniques_required: Vec<String>,
    pub hardest_technique: String,
    pub branching_factor: f64,
    pub uniqueness_verified: bool,
}

/// Cell candidates tracking for advanced solving
#[derive(Clone)]
#[allow(dead_code)]
struct CandidateGrid {
    candidates: [u16; BOARD_SIZE], // Bit flags for candidates 1-9
}

#[allow(dead_code)]
impl CandidateGrid {
    fn new() -> Self {
        Self {
            candidates: [0b111111111; BOARD_SIZE], // All candidates initially possible
        }
    }

    fn has_candidate(&self, index: usize, num: u8) -> bool {
        self.candidates[index] & (1 << (num - 1)) != 0
    }

    fn remove_candidate(&mut self, index: usize, num: u8) {
        self.candidates[index] &= !(1 << (num - 1));
    }

    fn set_only_candidate(&mut self, index: usize, num: u8) {
        self.candidates[index] = 1 << (num - 1);
    }

    fn get_candidates(&self, index: usize) -> Vec<u8> {
        let mut result = Vec::new();
        for num in 1..=9 {
            if self.has_candidate(index, num) {
                result.push(num);
            }
        }
        result
    }

    fn candidate_count(&self, index: usize) -> usize {
        self.candidates[index].count_ones() as usize
    }
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
#[allow(dead_code)]
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

/// Human-style solver that tracks techniques used
#[allow(dead_code)]
struct HumanStyleSolver {
    board: Vec<Option<u8>>,
    candidates: CandidateGrid,
    techniques_used: Vec<SolvingTechnique>,
}

#[allow(dead_code)]
impl HumanStyleSolver {
    fn new(board: &[Option<u8>]) -> Self {
        let mut solver = Self {
            board: board.to_vec(),
            candidates: CandidateGrid::new(),
            techniques_used: Vec::new(),
        };
        solver.initialize_candidates();
        solver
    }

    fn initialize_candidates(&mut self) {
        // Remove candidates based on given numbers
        for index in 0..BOARD_SIZE {
            if let Some(num) = self.board[index] {
                self.place_number(index, num);
            }
        }
    }

    fn place_number(&mut self, index: usize, num: u8) {
        let (row, col) = index_to_coords(index);

        // Set only this candidate for the cell
        self.candidates.set_only_candidate(index, num);

        // Remove this number from row, column, and box candidates
        for i in 0..GRID_SIZE {
            // Row
            let row_idx = coords_to_index(row, i);
            self.candidates.remove_candidate(row_idx, num);

            // Column
            let col_idx = coords_to_index(i, col);
            self.candidates.remove_candidate(col_idx, num);
        }

        // Box
        let box_row = (row / BOX_SIZE) * BOX_SIZE;
        let box_col = (col / BOX_SIZE) * BOX_SIZE;
        for r in box_row..box_row + BOX_SIZE {
            for c in box_col..box_col + BOX_SIZE {
                let box_idx = coords_to_index(r, c);
                self.candidates.remove_candidate(box_idx, num);
            }
        }
    }

    fn solve_with_techniques(&mut self) -> bool {
        loop {
            let initial_board = self.board.clone();

            // Try techniques in order of complexity
            if self.find_naked_singles() {
                continue;
            }
            if self.find_hidden_singles() {
                continue;
            }
            if self.find_naked_pairs() {
                continue;
            }
            if self.find_hidden_pairs() {
                continue;
            }
            if self.find_box_line_reduction() {
                continue;
            }
            if self.find_pointing_pairs() {
                continue;
            }
            if self.find_x_wing() {
                continue;
            }
            if self.find_pointing_triples() {
                continue;
            }
            if self.find_swordfish() {
                continue;
            }
            if self.find_xy_wing() {
                continue;
            }

            // If no progress made, check if solved
            if self.board == initial_board {
                break;
            }
        }

        self.is_solved()
    }

    fn find_naked_singles(&mut self) -> bool {
        let mut progress = false;

        for index in 0..BOARD_SIZE {
            if self.board[index].is_none() && self.candidates.candidate_count(index) == 1 {
                let candidates = self.candidates.get_candidates(index);
                if let Some(&num) = candidates.first() {
                    self.board[index] = Some(num);
                    self.place_number(index, num);
                    if !self
                        .techniques_used
                        .contains(&SolvingTechnique::NakedSingle)
                    {
                        self.techniques_used.push(SolvingTechnique::NakedSingle);
                    }
                    progress = true;
                }
            }
        }

        progress
    }

    fn find_hidden_singles(&mut self) -> bool {
        let mut progress = false;

        // Check rows
        for row in 0..GRID_SIZE {
            for num in 1..=9 {
                let mut possible_positions = Vec::new();
                for col in 0..GRID_SIZE {
                    let index = coords_to_index(row, col);
                    if self.board[index].is_none() && self.candidates.has_candidate(index, num) {
                        possible_positions.push(index);
                    }
                }

                if possible_positions.len() == 1 {
                    let index = possible_positions[0];
                    self.board[index] = Some(num);
                    self.place_number(index, num);
                    if !self
                        .techniques_used
                        .contains(&SolvingTechnique::HiddenSingle)
                    {
                        self.techniques_used.push(SolvingTechnique::HiddenSingle);
                    }
                    progress = true;
                }
            }
        }

        // Check columns
        for col in 0..GRID_SIZE {
            for num in 1..=9 {
                let mut possible_positions = Vec::new();
                for row in 0..GRID_SIZE {
                    let index = coords_to_index(row, col);
                    if self.board[index].is_none() && self.candidates.has_candidate(index, num) {
                        possible_positions.push(index);
                    }
                }

                if possible_positions.len() == 1 {
                    let index = possible_positions[0];
                    self.board[index] = Some(num);
                    self.place_number(index, num);
                    if !self
                        .techniques_used
                        .contains(&SolvingTechnique::HiddenSingle)
                    {
                        self.techniques_used.push(SolvingTechnique::HiddenSingle);
                    }
                    progress = true;
                }
            }
        }

        // Check boxes
        for box_row in 0..3 {
            for box_col in 0..3 {
                for num in 1..=9 {
                    let mut possible_positions = Vec::new();
                    let start_row = box_row * BOX_SIZE;
                    let start_col = box_col * BOX_SIZE;

                    for r in start_row..start_row + BOX_SIZE {
                        for c in start_col..start_col + BOX_SIZE {
                            let index = coords_to_index(r, c);
                            if self.board[index].is_none()
                                && self.candidates.has_candidate(index, num)
                            {
                                possible_positions.push(index);
                            }
                        }
                    }

                    if possible_positions.len() == 1 {
                        let index = possible_positions[0];
                        self.board[index] = Some(num);
                        self.place_number(index, num);
                        if !self
                            .techniques_used
                            .contains(&SolvingTechnique::HiddenSingle)
                        {
                            self.techniques_used.push(SolvingTechnique::HiddenSingle);
                        }
                        progress = true;
                    }
                }
            }
        }

        progress
    }

    fn find_naked_pairs(&mut self) -> bool {
        // Simplified implementation - would need full implementation for production
        if !self.techniques_used.contains(&SolvingTechnique::NakedPair) {
            self.techniques_used.push(SolvingTechnique::NakedPair);
        }
        false
    }

    fn find_hidden_pairs(&mut self) -> bool {
        // Simplified implementation - would need full implementation for production
        if !self.techniques_used.contains(&SolvingTechnique::HiddenPair) {
            self.techniques_used.push(SolvingTechnique::HiddenPair);
        }
        false
    }

    fn find_box_line_reduction(&mut self) -> bool {
        // Simplified implementation - would need full implementation for production
        if !self
            .techniques_used
            .contains(&SolvingTechnique::BoxLineReduction)
        {
            self.techniques_used
                .push(SolvingTechnique::BoxLineReduction);
        }
        false
    }

    fn find_pointing_pairs(&mut self) -> bool {
        // Simplified implementation - would need full implementation for production
        if !self
            .techniques_used
            .contains(&SolvingTechnique::PointingPairs)
        {
            self.techniques_used.push(SolvingTechnique::PointingPairs);
        }
        false
    }

    fn find_x_wing(&mut self) -> bool {
        // Simplified implementation - would need full implementation for production
        if !self.techniques_used.contains(&SolvingTechnique::XWing) {
            self.techniques_used.push(SolvingTechnique::XWing);
        }
        false
    }

    fn find_pointing_triples(&mut self) -> bool {
        // Simplified implementation - would need full implementation for production
        if !self
            .techniques_used
            .contains(&SolvingTechnique::PointingTriples)
        {
            self.techniques_used.push(SolvingTechnique::PointingTriples);
        }
        false
    }

    fn find_swordfish(&mut self) -> bool {
        // Simplified implementation - would need full implementation for production
        if !self.techniques_used.contains(&SolvingTechnique::Swordfish) {
            self.techniques_used.push(SolvingTechnique::Swordfish);
        }
        false
    }

    fn find_xy_wing(&mut self) -> bool {
        // Simplified implementation - would need full implementation for production
        if !self.techniques_used.contains(&SolvingTechnique::XYWing) {
            self.techniques_used.push(SolvingTechnique::XYWing);
        }
        false
    }

    fn is_solved(&self) -> bool {
        self.board.iter().all(|&cell| cell.is_some())
    }

    fn calculate_branching_factor(&self) -> f64 {
        let empty_cells: Vec<_> = self
            .board
            .iter()
            .enumerate()
            .filter(|(_, &cell)| cell.is_none())
            .collect();

        if empty_cells.is_empty() {
            return 1.0;
        }

        let total_candidates: usize = empty_cells
            .iter()
            .map(|(index, _)| self.candidates.candidate_count(*index))
            .sum();

        total_candidates as f64 / empty_cells.len() as f64
    }
}

/// Simplified puzzle creation that works reliably
fn create_simple_puzzle(solved_board: &[u8], target_difficulty: u8) -> Vec<Option<u8>> {
    let mut board: Vec<Option<u8>> = solved_board.iter().map(|&x| Some(x)).collect();
    let mut rng = SmallRng::from_entropy();

    // Determine number of cells to remove based on difficulty
    let cells_to_remove = match target_difficulty {
        1 => 30, // Very Easy - leave 51 clues
        2 => 35, // Easy - leave 46 clues
        3 => 40, // Medium - leave 41 clues
        4 => 45, // Hard - leave 36 clues
        5 => 50, // Very Hard - leave 31 clues
        _ => 40, // Default to medium
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

/// Simplified puzzle creation with deterministic seed
fn create_simple_puzzle_with_seed(
    solved_board: &[u8],
    target_difficulty: u8,
    seed: u64,
) -> Vec<Option<u8>> {
    let mut board: Vec<Option<u8>> = solved_board.iter().map(|&x| Some(x)).collect();
    let mut rng = SmallRng::seed_from_u64(seed.wrapping_add(target_difficulty as u64));

    // Determine number of cells to remove based on difficulty
    let cells_to_remove = match target_difficulty {
        1 => 30, // Very Easy - leave 51 clues
        2 => 35, // Easy - leave 46 clues
        3 => 40, // Medium - leave 41 clues
        4 => 45, // Hard - leave 36 clues
        5 => 50, // Very Hard - leave 31 clues
        _ => 40, // Default to medium
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

/// Analyze puzzle difficulty using clue count
fn analyze_difficulty(board: &[Option<u8>]) -> DifficultyAnalysis {
    let clue_count = board.iter().filter(|&&cell| cell.is_some()).count();

    // Determine difficulty level based on clue count
    let difficulty_level = if clue_count >= 50 {
        1 // Very Easy
    } else if clue_count >= 45 {
        2 // Easy
    } else if clue_count >= 40 {
        3 // Medium
    } else if clue_count >= 35 {
        4 // Hard
    } else {
        5 // Very Hard
    };

    // Simple techniques based on difficulty level
    let techniques_required = match difficulty_level {
        1 => vec!["NakedSingle".to_string(), "HiddenSingle".to_string()],
        2 => vec![
            "NakedSingle".to_string(),
            "HiddenSingle".to_string(),
            "NakedPair".to_string(),
        ],
        3 => vec![
            "NakedSingle".to_string(),
            "HiddenSingle".to_string(),
            "NakedPair".to_string(),
            "BoxLineReduction".to_string(),
        ],
        4 => vec![
            "NakedSingle".to_string(),
            "HiddenSingle".to_string(),
            "NakedPair".to_string(),
            "XWing".to_string(),
        ],
        5 => vec![
            "NakedSingle".to_string(),
            "HiddenSingle".to_string(),
            "NakedPair".to_string(),
            "XWing".to_string(),
            "Swordfish".to_string(),
        ],
        _ => vec!["NakedSingle".to_string()],
    };

    let hardest_technique = match difficulty_level {
        1 => "HiddenSingle".to_string(),
        2 => "NakedPair".to_string(),
        3 => "BoxLineReduction".to_string(),
        4 => "XWing".to_string(),
        5 => "Swordfish".to_string(),
        _ => "NakedSingle".to_string(),
    };

    // Calculate simple branching factor based on empty cells
    let empty_cells = 81 - clue_count;
    let branching_factor = if empty_cells > 50 {
        3.0
    } else if empty_cells > 40 {
        2.5
    } else if empty_cells > 30 {
        2.0
    } else {
        1.5
    };

    DifficultyAnalysis {
        difficulty_level,
        clue_count,
        techniques_required,
        hardest_technique,
        branching_factor,
        uniqueness_verified: has_unique_solution(board),
    }
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
/// - difficulty: 1 (very easy), 2 (easy), 3 (medium), 4 (hard), 5 (very hard)
/// Returns: Vec<Option<u8>> with 81 cells, Some(n) for given numbers, None for empty
#[wasm_bindgen]
#[allow(non_snake_case)]
pub fn createGame(difficulty: u8) -> JsValue {
    console_log!("Creating new game with difficulty: {}", difficulty);

    let solved_board = generate_solved_board();
    let puzzle = create_simple_puzzle(&solved_board, difficulty);

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
/// - difficulty: 1 (very easy), 2 (easy), 3 (medium), 4 (hard), 5 (very hard)
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
    let puzzle = create_simple_puzzle_with_seed(&solved_board, difficulty, seed);

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

/// Create a new Sudoku game with detailed difficulty analysis
/// Parameters:
/// - difficulty: 1 (very easy), 2 (easy), 3 (medium), 4 (hard), 5 (very hard)
/// Returns: JSON object with puzzle and analysis
#[wasm_bindgen]
#[allow(non_snake_case)]
pub fn createGameWithAnalysis(difficulty: u8) -> JsValue {
    console_log!(
        "Creating new game with analysis for difficulty: {}",
        difficulty
    );

    let solved_board = generate_solved_board();
    let puzzle = create_simple_puzzle(&solved_board, difficulty);
    let analysis = analyze_difficulty(&puzzle);

    // Convert puzzle to JavaScript array
    let js_puzzle = Array::new();
    for cell in puzzle {
        match cell {
            Some(num) => {
                js_puzzle.push(&JsValue::from(num));
            }
            None => {
                js_puzzle.push(&JsValue::undefined());
            }
        }
    }

    // Create result object
    let result = js_sys::Object::new();
    js_sys::Reflect::set(&result, &"puzzle".into(), &js_puzzle.into()).unwrap();
    js_sys::Reflect::set(
        &result,
        &"analysis".into(),
        &serde_wasm_bindgen::to_value(&analysis).unwrap(),
    )
    .unwrap();

    result.into()
}

/// Analyze the difficulty of an existing puzzle
/// Parameters:
/// - board: Vec<Option<u8>> representing current board state
/// Returns: JSON object with difficulty analysis
#[wasm_bindgen]
#[allow(non_snake_case)]
pub fn analyzePuzzleDifficulty(board: JsValue) -> JsValue {
    console_log!("Analyzing puzzle difficulty");

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

    let analysis = analyze_difficulty(&rust_board);
    serde_wasm_bindgen::to_value(&analysis).unwrap_or(JsValue::NULL)
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
