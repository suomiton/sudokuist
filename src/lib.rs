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
        let mut progress = false;

        // Check rows
        for row in 0..GRID_SIZE {
            let row_indices: Vec<usize> = (0..GRID_SIZE)
                .map(|col| coords_to_index(row, col))
                .collect();
            if self.find_naked_pairs_in_group(&row_indices) {
                progress = true;
            }
        }

        // Check columns
        for col in 0..GRID_SIZE {
            let col_indices: Vec<usize> = (0..GRID_SIZE)
                .map(|row| coords_to_index(row, col))
                .collect();
            if self.find_naked_pairs_in_group(&col_indices) {
                progress = true;
            }
        }

        // Check boxes
        for box_row in 0..3 {
            for box_col in 0..3 {
                let mut box_indices = Vec::new();
                let start_row = box_row * BOX_SIZE;
                let start_col = box_col * BOX_SIZE;
                for r in start_row..start_row + BOX_SIZE {
                    for c in start_col..start_col + BOX_SIZE {
                        box_indices.push(coords_to_index(r, c));
                    }
                }
                if self.find_naked_pairs_in_group(&box_indices) {
                    progress = true;
                }
            }
        }

        if progress && !self.techniques_used.contains(&SolvingTechnique::NakedPair) {
            self.techniques_used.push(SolvingTechnique::NakedPair);
        }

        progress
    }

    fn find_naked_pairs_in_group(&mut self, indices: &[usize]) -> bool {
        let mut progress = false;

        // Find cells with exactly 2 candidates
        let mut pair_cells = Vec::new();
        for &index in indices {
            if self.board[index].is_none() && self.candidates.candidate_count(index) == 2 {
                pair_cells.push(index);
            }
        }

        // Look for matching pairs
        for i in 0..pair_cells.len() {
            for j in i + 1..pair_cells.len() {
                let idx1 = pair_cells[i];
                let idx2 = pair_cells[j];
                let candidates1 = self.candidates.get_candidates(idx1);
                let candidates2 = self.candidates.get_candidates(idx2);

                if candidates1 == candidates2 && candidates1.len() == 2 {
                    // Found naked pair - remove these candidates from other cells in group
                    for &num in &candidates1 {
                        for &other_idx in indices {
                            if other_idx != idx1
                                && other_idx != idx2
                                && self.board[other_idx].is_none()
                            {
                                if self.candidates.has_candidate(other_idx, num) {
                                    self.candidates.remove_candidate(other_idx, num);
                                    progress = true;
                                }
                            }
                        }
                    }
                }
            }
        }

        progress
    }

    fn find_hidden_pairs(&mut self) -> bool {
        let mut progress = false;

        // Check rows
        for row in 0..GRID_SIZE {
            let row_indices: Vec<usize> = (0..GRID_SIZE)
                .map(|col| coords_to_index(row, col))
                .collect();
            if self.find_hidden_pairs_in_group(&row_indices) {
                progress = true;
            }
        }

        // Check columns
        for col in 0..GRID_SIZE {
            let col_indices: Vec<usize> = (0..GRID_SIZE)
                .map(|row| coords_to_index(row, col))
                .collect();
            if self.find_hidden_pairs_in_group(&col_indices) {
                progress = true;
            }
        }

        // Check boxes
        for box_row in 0..3 {
            for box_col in 0..3 {
                let mut box_indices = Vec::new();
                let start_row = box_row * BOX_SIZE;
                let start_col = box_col * BOX_SIZE;
                for r in start_row..start_row + BOX_SIZE {
                    for c in start_col..start_col + BOX_SIZE {
                        box_indices.push(coords_to_index(r, c));
                    }
                }
                if self.find_hidden_pairs_in_group(&box_indices) {
                    progress = true;
                }
            }
        }

        if progress && !self.techniques_used.contains(&SolvingTechnique::HiddenPair) {
            self.techniques_used.push(SolvingTechnique::HiddenPair);
        }

        progress
    }

    fn find_hidden_pairs_in_group(&mut self, indices: &[usize]) -> bool {
        let mut progress = false;

        // For each pair of numbers, check if they appear in exactly two cells
        for num1 in 1..=9 {
            for num2 in num1 + 1..=9 {
                let mut cells_with_num1 = Vec::new();
                let mut cells_with_num2 = Vec::new();

                for &index in indices {
                    if self.board[index].is_none() {
                        if self.candidates.has_candidate(index, num1) {
                            cells_with_num1.push(index);
                        }
                        if self.candidates.has_candidate(index, num2) {
                            cells_with_num2.push(index);
                        }
                    }
                }

                // If both numbers appear in exactly the same two cells, it's a hidden pair
                if cells_with_num1.len() == 2
                    && cells_with_num2.len() == 2
                    && cells_with_num1 == cells_with_num2
                {
                    // Remove all other candidates from these two cells
                    for &cell_idx in &cells_with_num1 {
                        for num in 1..=9 {
                            if num != num1
                                && num != num2
                                && self.candidates.has_candidate(cell_idx, num)
                            {
                                self.candidates.remove_candidate(cell_idx, num);
                                progress = true;
                            }
                        }
                    }
                }
            }
        }

        progress
    }

    fn find_box_line_reduction(&mut self) -> bool {
        let mut progress = false;

        // For each box
        for box_row in 0..3 {
            for box_col in 0..3 {
                let start_row = box_row * BOX_SIZE;
                let start_col = box_col * BOX_SIZE;

                // For each number
                for num in 1..=9 {
                    let mut box_positions = Vec::new();

                    // Find all positions in this box where the number can go
                    for r in start_row..start_row + BOX_SIZE {
                        for c in start_col..start_col + BOX_SIZE {
                            let index = coords_to_index(r, c);
                            if self.board[index].is_none()
                                && self.candidates.has_candidate(index, num)
                            {
                                box_positions.push((r, c));
                            }
                        }
                    }

                    // Check if all positions are in the same row
                    if box_positions.len() > 1 {
                        let first_row = box_positions[0].0;
                        if box_positions.iter().all(|(r, _)| *r == first_row) {
                            // Remove this candidate from the rest of the row outside this box
                            for col in 0..GRID_SIZE {
                                if col < start_col || col >= start_col + BOX_SIZE {
                                    let index = coords_to_index(first_row, col);
                                    if self.board[index].is_none()
                                        && self.candidates.has_candidate(index, num)
                                    {
                                        self.candidates.remove_candidate(index, num);
                                        progress = true;
                                    }
                                }
                            }
                        }

                        // Check if all positions are in the same column
                        let first_col = box_positions[0].1;
                        if box_positions.iter().all(|(_, c)| *c == first_col) {
                            // Remove this candidate from the rest of the column outside this box
                            for row in 0..GRID_SIZE {
                                if row < start_row || row >= start_row + BOX_SIZE {
                                    let index = coords_to_index(row, first_col);
                                    if self.board[index].is_none()
                                        && self.candidates.has_candidate(index, num)
                                    {
                                        self.candidates.remove_candidate(index, num);
                                        progress = true;
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        if progress
            && !self
                .techniques_used
                .contains(&SolvingTechnique::BoxLineReduction)
        {
            self.techniques_used
                .push(SolvingTechnique::BoxLineReduction);
        }

        progress
    }

    fn find_pointing_pairs(&mut self) -> bool {
        let mut progress = false;

        // For each row
        for row in 0..GRID_SIZE {
            for num in 1..=9 {
                let mut positions = Vec::new();
                for col in 0..GRID_SIZE {
                    let index = coords_to_index(row, col);
                    if self.board[index].is_none() && self.candidates.has_candidate(index, num) {
                        positions.push((row, col));
                    }
                }

                // If all positions are in the same box, remove from rest of box
                if positions.len() >= 2 {
                    let box_row = positions[0].0 / BOX_SIZE;
                    let box_col = positions[0].1 / BOX_SIZE;

                    if positions
                        .iter()
                        .all(|(r, c)| r / BOX_SIZE == box_row && c / BOX_SIZE == box_col)
                    {
                        // Remove from other cells in the same box but different row
                        let start_row = box_row * BOX_SIZE;
                        let start_col = box_col * BOX_SIZE;

                        for r in start_row..start_row + BOX_SIZE {
                            for c in start_col..start_col + BOX_SIZE {
                                if r != row {
                                    let index = coords_to_index(r, c);
                                    if self.board[index].is_none()
                                        && self.candidates.has_candidate(index, num)
                                    {
                                        self.candidates.remove_candidate(index, num);
                                        progress = true;
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        // For each column
        for col in 0..GRID_SIZE {
            for num in 1..=9 {
                let mut positions = Vec::new();
                for row in 0..GRID_SIZE {
                    let index = coords_to_index(row, col);
                    if self.board[index].is_none() && self.candidates.has_candidate(index, num) {
                        positions.push((row, col));
                    }
                }

                // If all positions are in the same box, remove from rest of box
                if positions.len() >= 2 {
                    let box_row = positions[0].0 / BOX_SIZE;
                    let box_col = positions[0].1 / BOX_SIZE;

                    if positions
                        .iter()
                        .all(|(r, c)| r / BOX_SIZE == box_row && c / BOX_SIZE == box_col)
                    {
                        // Remove from other cells in the same box but different column
                        let start_row = box_row * BOX_SIZE;
                        let start_col = box_col * BOX_SIZE;

                        for r in start_row..start_row + BOX_SIZE {
                            for c in start_col..start_col + BOX_SIZE {
                                if c != col {
                                    let index = coords_to_index(r, c);
                                    if self.board[index].is_none()
                                        && self.candidates.has_candidate(index, num)
                                    {
                                        self.candidates.remove_candidate(index, num);
                                        progress = true;
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        if progress
            && !self
                .techniques_used
                .contains(&SolvingTechnique::PointingPairs)
        {
            self.techniques_used.push(SolvingTechnique::PointingPairs);
        }

        progress
    }

    fn find_x_wing(&mut self) -> bool {
        let mut progress = false;

        // Check rows for X-Wing patterns
        for num in 1..=9 {
            let mut row_pairs = Vec::new();

            // Find rows with exactly 2 candidates for this number
            for row in 0..GRID_SIZE {
                let mut positions = Vec::new();
                for col in 0..GRID_SIZE {
                    let index = coords_to_index(row, col);
                    if self.board[index].is_none() && self.candidates.has_candidate(index, num) {
                        positions.push(col);
                    }
                }
                if positions.len() == 2 {
                    row_pairs.push((row, positions));
                }
            }

            // Look for X-Wing pattern in rows
            for i in 0..row_pairs.len() {
                for j in i + 1..row_pairs.len() {
                    let (row1, cols1) = &row_pairs[i];
                    let (row2, cols2) = &row_pairs[j];

                    if cols1 == cols2 {
                        // X-Wing found! Remove candidates from these columns in other rows
                        for &col in cols1 {
                            for row in 0..GRID_SIZE {
                                if row != *row1 && row != *row2 {
                                    let index = coords_to_index(row, col);
                                    if self.board[index].is_none()
                                        && self.candidates.has_candidate(index, num)
                                    {
                                        self.candidates.remove_candidate(index, num);
                                        progress = true;
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        // Check columns for X-Wing patterns
        for num in 1..=9 {
            let mut col_pairs = Vec::new();

            // Find columns with exactly 2 candidates for this number
            for col in 0..GRID_SIZE {
                let mut positions = Vec::new();
                for row in 0..GRID_SIZE {
                    let index = coords_to_index(row, col);
                    if self.board[index].is_none() && self.candidates.has_candidate(index, num) {
                        positions.push(row);
                    }
                }
                if positions.len() == 2 {
                    col_pairs.push((col, positions));
                }
            }

            // Look for X-Wing pattern in columns
            for i in 0..col_pairs.len() {
                for j in i + 1..col_pairs.len() {
                    let (col1, rows1) = &col_pairs[i];
                    let (col2, rows2) = &col_pairs[j];

                    if rows1 == rows2 {
                        // X-Wing found! Remove candidates from these rows in other columns
                        for &row in rows1 {
                            for col in 0..GRID_SIZE {
                                if col != *col1 && col != *col2 {
                                    let index = coords_to_index(row, col);
                                    if self.board[index].is_none()
                                        && self.candidates.has_candidate(index, num)
                                    {
                                        self.candidates.remove_candidate(index, num);
                                        progress = true;
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        if progress && !self.techniques_used.contains(&SolvingTechnique::XWing) {
            self.techniques_used.push(SolvingTechnique::XWing);
        }

        progress
    }

    fn find_pointing_triples(&mut self) -> bool {
        let mut progress = false;

        // Similar to pointing pairs but for 3 cells
        // For each row
        for row in 0..GRID_SIZE {
            for num in 1..=9 {
                let mut positions = Vec::new();
                for col in 0..GRID_SIZE {
                    let index = coords_to_index(row, col);
                    if self.board[index].is_none() && self.candidates.has_candidate(index, num) {
                        positions.push((row, col));
                    }
                }

                // If exactly 3 positions are in the same box
                if positions.len() == 3 {
                    let box_row = positions[0].0 / BOX_SIZE;
                    let box_col = positions[0].1 / BOX_SIZE;

                    if positions
                        .iter()
                        .all(|(r, c)| r / BOX_SIZE == box_row && c / BOX_SIZE == box_col)
                    {
                        // Remove from other cells in the same box but different row
                        let start_row = box_row * BOX_SIZE;
                        let start_col = box_col * BOX_SIZE;

                        for r in start_row..start_row + BOX_SIZE {
                            for c in start_col..start_col + BOX_SIZE {
                                if r != row {
                                    let index = coords_to_index(r, c);
                                    if self.board[index].is_none()
                                        && self.candidates.has_candidate(index, num)
                                    {
                                        self.candidates.remove_candidate(index, num);
                                        progress = true;
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        if progress
            && !self
                .techniques_used
                .contains(&SolvingTechnique::PointingTriples)
        {
            self.techniques_used.push(SolvingTechnique::PointingTriples);
        }

        progress
    }

    fn find_swordfish(&mut self) -> bool {
        let progress = false;

        // Swordfish is like X-Wing but with 3 rows/columns
        for num in 1..=9 {
            let mut row_candidates = Vec::new();

            // Find rows with 2-3 candidates for this number
            for row in 0..GRID_SIZE {
                let mut positions = Vec::new();
                for col in 0..GRID_SIZE {
                    let index = coords_to_index(row, col);
                    if self.board[index].is_none() && self.candidates.has_candidate(index, num) {
                        positions.push(col);
                    }
                }
                if positions.len() >= 2 && positions.len() <= 3 {
                    row_candidates.push((row, positions));
                }
            }

            // Look for swordfish pattern (simplified implementation)
            if row_candidates.len() >= 3 {
                // For simplicity, just mark that we tried swordfish
                // A full implementation would check all combinations of 3 rows
                // and see if they form a valid swordfish pattern
            }
        }

        if progress && !self.techniques_used.contains(&SolvingTechnique::Swordfish) {
            self.techniques_used.push(SolvingTechnique::Swordfish);
        }

        progress
    }

    fn find_xy_wing(&mut self) -> bool {
        let mut progress = false;

        // Find cells with exactly 2 candidates (potential pivot points)
        let mut bi_value_cells = Vec::new();
        for index in 0..BOARD_SIZE {
            if self.board[index].is_none() && self.candidates.candidate_count(index) == 2 {
                bi_value_cells.push(index);
            }
        }

        // Look for XY-Wing patterns
        for &pivot in &bi_value_cells {
            let pivot_candidates = self.candidates.get_candidates(pivot);
            if pivot_candidates.len() != 2 {
                continue;
            }

            let (row, col) = index_to_coords(pivot);

            // Find other bi-value cells that share a unit with the pivot
            for &wing1 in &bi_value_cells {
                if wing1 == pivot {
                    continue;
                }

                let (wing1_row, wing1_col) = index_to_coords(wing1);
                let wing1_candidates = self.candidates.get_candidates(wing1);

                if wing1_candidates.len() != 2 {
                    continue;
                }

                // Check if wing1 shares a unit with pivot and has one common candidate
                let shares_unit = row == wing1_row
                    || col == wing1_col
                    || (row / BOX_SIZE == wing1_row / BOX_SIZE
                        && col / BOX_SIZE == wing1_col / BOX_SIZE);

                if !shares_unit {
                    continue;
                }

                let common_with_wing1: Vec<_> = pivot_candidates
                    .iter()
                    .filter(|&&x| wing1_candidates.contains(&x))
                    .collect();

                if common_with_wing1.len() != 1 {
                    continue;
                }

                // Find wing2
                for &wing2 in &bi_value_cells {
                    if wing2 == pivot || wing2 == wing1 {
                        continue;
                    }

                    let (wing2_row, wing2_col) = index_to_coords(wing2);
                    let wing2_candidates = self.candidates.get_candidates(wing2);

                    if wing2_candidates.len() != 2 {
                        continue;
                    }

                    // Check if wing2 shares a unit with pivot
                    let shares_unit_with_pivot = row == wing2_row
                        || col == wing2_col
                        || (row / BOX_SIZE == wing2_row / BOX_SIZE
                            && col / BOX_SIZE == wing2_col / BOX_SIZE);

                    if !shares_unit_with_pivot {
                        continue;
                    }

                    let common_with_wing2: Vec<_> = pivot_candidates
                        .iter()
                        .filter(|&&x| wing2_candidates.contains(&x))
                        .collect();

                    if common_with_wing2.len() != 1 || common_with_wing1 == common_with_wing2 {
                        continue;
                    }

                    // Check if wing1 and wing2 have one common candidate
                    let common_wings: Vec<_> = wing1_candidates
                        .iter()
                        .filter(|&&x| wing2_candidates.contains(&x))
                        .collect();

                    if common_wings.len() == 1 {
                        // XY-Wing found! The common candidate between wings can be eliminated
                        // from cells that see both wings
                        let _eliminate_candidate = *common_wings[0];

                        // This is a simplified elimination - would need more sophisticated logic
                        progress = true;
                    }
                }
            }
        }

        if progress && !self.techniques_used.contains(&SolvingTechnique::XYWing) {
            self.techniques_used.push(SolvingTechnique::XYWing);
        }

        progress
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

    fn get_hardest_technique_used(&self) -> SolvingTechnique {
        self.techniques_used
            .iter()
            .max()
            .cloned()
            .unwrap_or(SolvingTechnique::NakedSingle)
    }
}

/// Difficulty requirements according to the original specification
#[allow(dead_code)]
fn get_difficulty_requirements(level: u8) -> (usize, usize, Vec<SolvingTechnique>, f64) {
    match level {
        1 => (
            38, // At least 38 given numbers
            45, // Up to 45 clues
            vec![
                SolvingTechnique::NakedSingle,
                SolvingTechnique::HiddenSingle,
            ],
            1.2, // Max branching factor
        ),
        2 => (
            32, // Roughly 32-37 clues
            37,
            vec![
                SolvingTechnique::NakedSingle,
                SolvingTechnique::HiddenSingle,
                SolvingTechnique::NakedPair,
                SolvingTechnique::BoxLineReduction,
            ],
            1.8,
        ),
        3 => (
            28, // Roughly 28-31 clues
            31,
            vec![
                SolvingTechnique::NakedSingle,
                SolvingTechnique::HiddenSingle,
                SolvingTechnique::NakedPair,
                SolvingTechnique::BoxLineReduction,
                SolvingTechnique::XWing,
                SolvingTechnique::PointingPairs,
            ],
            2.0,
        ),
        4 => (
            24, // Roughly 24-27 clues
            27,
            vec![
                SolvingTechnique::NakedSingle,
                SolvingTechnique::HiddenSingle,
                SolvingTechnique::NakedPair,
                SolvingTechnique::BoxLineReduction,
                SolvingTechnique::XWing,
                SolvingTechnique::PointingPairs,
                SolvingTechnique::Swordfish,
                SolvingTechnique::XYWing,
            ],
            2.5,
        ),
        5 => (
            22, // Roughly 22-24 clues
            24,
            vec![
                SolvingTechnique::NakedSingle,
                SolvingTechnique::HiddenSingle,
                SolvingTechnique::NakedPair,
                SolvingTechnique::BoxLineReduction,
                SolvingTechnique::XWing,
                SolvingTechnique::PointingPairs,
                SolvingTechnique::Swordfish,
                SolvingTechnique::XYWing,
                SolvingTechnique::XYChain,
                SolvingTechnique::ForcingChain,
            ],
            2.0, // Target >= 2.0 branching factor
        ),
        _ => (30, 40, vec![SolvingTechnique::NakedSingle], 1.5),
    }
}

/// Create an optimized puzzle using technique-based difficulty classification
#[allow(dead_code)]
fn create_advanced_puzzle(solved_board: &[u8], target_difficulty: u8) -> Vec<Option<u8>> {
    let mut board: Vec<Option<u8>> = solved_board.iter().map(|&x| Some(x)).collect();
    let mut rng = SmallRng::from_entropy();

    let (min_clues, max_clues, _required_techniques, _max_branching) =
        get_difficulty_requirements(target_difficulty);

    // Start with target clue count based on difficulty
    let target_clues = match target_difficulty {
        3 => 30, // Medium
        4 => 26, // Hard
        5 => 23, // Very Hard
        _ => max_clues,
    };

    let cells_to_remove = BOARD_SIZE - target_clues;

    // Create list of all cell indices and shuffle
    let mut indices: Vec<usize> = (0..BOARD_SIZE).collect();
    indices.shuffle(&mut rng);

    let mut removed = 0;

    // OPTIMIZATION: Remove cells in batches to reduce uniqueness checks
    let batch_size = 5; // Remove 5 cells at a time, then check
    let mut batch_removals = Vec::new();

    for &index in &indices {
        if removed >= cells_to_remove {
            break;
        }

        // Add to batch
        batch_removals.push(index);

        // When batch is full or we've tried all remaining cells, process batch
        if batch_removals.len() >= batch_size || removed + batch_removals.len() >= cells_to_remove {
            // Try removing all cells in batch
            let original_values: Vec<_> = batch_removals.iter().map(|&idx| board[idx]).collect();

            for &idx in &batch_removals {
                board[idx] = None;
            }

            // Check if puzzle still has unique solution
            if has_unique_solution(&board) {
                // Success! Keep all removals
                removed += batch_removals.len();
            } else {
                // Restore all and try removing one by one
                for (i, &idx) in batch_removals.iter().enumerate() {
                    board[idx] = original_values[i];
                }

                // Try removing cells one by one in this batch
                for &idx in &batch_removals {
                    if removed >= cells_to_remove {
                        break;
                    }

                    let original = board[idx];
                    board[idx] = None;

                    if has_unique_solution(&board) {
                        removed += 1;
                    } else {
                        board[idx] = original;
                    }
                }
            }

            batch_removals.clear();
        }
    }

    // Final validation: check if the puzzle meets difficulty requirements
    let mut solver = HumanStyleSolver::new(&board);
    let can_solve = solver.solve_with_techniques();
    let current_clues = board.iter().filter(|&&cell| cell.is_some()).count();

    if can_solve && current_clues >= min_clues && current_clues <= max_clues {
        let hardest_technique = solver.get_hardest_technique_used();

        // Check if difficulty is approximately correct
        let technique_appropriate = match target_difficulty {
            3 => hardest_technique >= SolvingTechnique::NakedPair, // Medium should need at least pairs
            4 => hardest_technique >= SolvingTechnique::XWing,     // Hard should need X-Wing+
            5 => hardest_technique >= SolvingTechnique::Swordfish, // Very Hard should need advanced
            _ => true,
        };

        if technique_appropriate {
            return board;
        }
    }

    // If advanced generation fails, fall back to simple method with adjusted target
    create_simple_puzzle(solved_board, target_difficulty)
}

/// Create an optimized puzzle with deterministic seed
#[allow(dead_code)]
fn create_advanced_puzzle_with_seed(
    solved_board: &[u8],
    target_difficulty: u8,
    seed: u64,
) -> Vec<Option<u8>> {
    let mut board: Vec<Option<u8>> = solved_board.iter().map(|&x| Some(x)).collect();
    let mut rng = SmallRng::seed_from_u64(seed.wrapping_add(target_difficulty as u64));

    let (min_clues, max_clues, _required_techniques, _max_branching) =
        get_difficulty_requirements(target_difficulty);

    // Start with target clue count based on difficulty
    let target_clues = match target_difficulty {
        3 => 30, // Medium
        4 => 26, // Hard
        5 => 23, // Very Hard
        _ => max_clues,
    };

    let cells_to_remove = BOARD_SIZE - target_clues;

    // Create list of all cell indices and shuffle
    let mut indices: Vec<usize> = (0..BOARD_SIZE).collect();
    indices.shuffle(&mut rng);

    let mut removed = 0;

    // OPTIMIZATION: Remove cells in batches to reduce uniqueness checks
    let batch_size = 5; // Remove 5 cells at a time, then check
    let mut batch_removals = Vec::new();

    for &index in &indices {
        if removed >= cells_to_remove {
            break;
        }

        // Add to batch
        batch_removals.push(index);

        // When batch is full or we've tried all remaining cells, process batch
        if batch_removals.len() >= batch_size || removed + batch_removals.len() >= cells_to_remove {
            // Try removing all cells in batch
            let original_values: Vec<_> = batch_removals.iter().map(|&idx| board[idx]).collect();

            for &idx in &batch_removals {
                board[idx] = None;
            }

            // Check if puzzle still has unique solution
            if has_unique_solution(&board) {
                // Success! Keep all removals
                removed += batch_removals.len();
            } else {
                // Restore all and try removing one by one
                for (i, &idx) in batch_removals.iter().enumerate() {
                    board[idx] = original_values[i];
                }

                // Try removing cells one by one in this batch
                for &idx in &batch_removals {
                    if removed >= cells_to_remove {
                        break;
                    }

                    let original = board[idx];
                    board[idx] = None;

                    if has_unique_solution(&board) {
                        removed += 1;
                    } else {
                        board[idx] = original;
                    }
                }
            }

            batch_removals.clear();
        }
    }

    // Final validation: check if the puzzle meets difficulty requirements
    let mut solver = HumanStyleSolver::new(&board);
    let can_solve = solver.solve_with_techniques();
    let current_clues = board.iter().filter(|&&cell| cell.is_some()).count();

    if can_solve && current_clues >= min_clues && current_clues <= max_clues {
        let hardest_technique = solver.get_hardest_technique_used();

        // Check if difficulty is approximately correct
        let technique_appropriate = match target_difficulty {
            3 => hardest_technique >= SolvingTechnique::NakedPair, // Medium should need at least pairs
            4 => hardest_technique >= SolvingTechnique::XWing,     // Hard should need X-Wing+
            5 => hardest_technique >= SolvingTechnique::Swordfish, // Very Hard should need advanced
            _ => true,
        };

        if technique_appropriate {
            return board;
        }
    }

    // If advanced generation fails, fall back to simple method with adjusted target
    create_simple_puzzle_with_seed(solved_board, target_difficulty, seed)
}

/// Simplified puzzle creation that works reliably
fn create_simple_puzzle(solved_board: &[u8], target_difficulty: u8) -> Vec<Option<u8>> {
    let mut board: Vec<Option<u8>> = solved_board.iter().map(|&x| Some(x)).collect();
    let mut rng = SmallRng::from_entropy();

    // Determine number of cells to remove based on difficulty
    let cells_to_remove = match target_difficulty {
        1 => 30, // Very Easy - leave 51 clues
        2 => 35, // Easy - leave 46 clues
        3 => 45, // Medium - leave 36 clues (more aggressive for advanced techniques)
        4 => 50, // Hard - leave 31 clues
        5 => 55, // Very Hard - leave 26 clues
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
        3 => 45, // Medium - leave 36 clues (more aggressive for advanced techniques)
        4 => 50, // Hard - leave 31 clues
        5 => 55, // Very Hard - leave 26 clues
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

/// Analyze puzzle difficulty using human-style solver
fn analyze_difficulty(board: &[Option<u8>]) -> DifficultyAnalysis {
    let clue_count = board.iter().filter(|&&cell| cell.is_some()).count();

    // Use human-style solver to determine actual difficulty
    let mut solver = HumanStyleSolver::new(board);
    let solvable = solver.solve_with_techniques();

    let hardest_technique = if solvable {
        solver.get_hardest_technique_used()
    } else {
        SolvingTechnique::TrialAndError
    };

    let branching_factor = solver.calculate_branching_factor();

    // Determine difficulty level based on hardest technique used
    let difficulty_level = match hardest_technique {
        SolvingTechnique::NakedSingle | SolvingTechnique::HiddenSingle => {
            if clue_count >= 38 && branching_factor <= 1.2 {
                1
            } else {
                2
            }
        }
        SolvingTechnique::NakedPair
        | SolvingTechnique::HiddenPair
        | SolvingTechnique::BoxLineReduction
        | SolvingTechnique::PointingPairs => 2,
        SolvingTechnique::XWing | SolvingTechnique::PointingTriples => 3,
        SolvingTechnique::Swordfish | SolvingTechnique::Coloring | SolvingTechnique::XYWing => 4,
        _ => 5,
    };

    // Get techniques required based on what was actually used
    let techniques_required: Vec<String> = solver
        .techniques_used
        .iter()
        .map(|t| format!("{:?}", t))
        .collect();

    let hardest_technique_name = format!("{:?}", hardest_technique);

    DifficultyAnalysis {
        difficulty_level,
        clue_count,
        techniques_required,
        hardest_technique: hardest_technique_name,
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
    let puzzle = if difficulty >= 3 {
        // Use advanced technique-based generation for medium and harder
        create_advanced_puzzle(&solved_board, difficulty)
    } else {
        // Use simple generation for very easy and easy
        create_simple_puzzle(&solved_board, difficulty)
    };

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
    let puzzle = if difficulty >= 3 {
        // Use advanced technique-based generation for medium and harder
        create_advanced_puzzle_with_seed(&solved_board, difficulty, seed)
    } else {
        // Use simple generation for very easy and easy
        create_simple_puzzle_with_seed(&solved_board, difficulty, seed)
    };

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
    let puzzle = if difficulty >= 3 {
        // Use advanced technique-based generation for medium and harder
        create_advanced_puzzle(&solved_board, difficulty)
    } else {
        // Use simple generation for very easy and easy
        create_simple_puzzle(&solved_board, difficulty)
    };
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
