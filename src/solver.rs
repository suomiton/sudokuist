//! Human-style Sudoku solving techniques
//!
//! This module implements a solver that mimics human logical reasoning
//! by applying various solving techniques in order of increasing complexity.

use crate::types::{SolvingTechnique, CandidateGrid, BOARD_SIZE, GRID_SIZE, BOX_SIZE};
use crate::grid::{index_to_coords, coords_to_index};

/// A human-style Sudoku solver that tracks which techniques are used
///
/// This solver attempts to solve puzzles using logical techniques that
/// humans would use, rather than brute-force backtracking. It maintains
/// a candidate grid and applies techniques in order of increasing difficulty.
pub struct HumanStyleSolver {
    /// The current state of the board
    board: Vec<Option<u8>>,
    /// Tracks possible candidates for each empty cell
    candidates: CandidateGrid,
    /// List of techniques that were used during solving
    techniques_used: Vec<SolvingTechnique>,
}

impl HumanStyleSolver {
    /// Creates a new solver for the given board
    ///
    /// # Arguments
    /// * `board` - The initial board state with clues
    ///
    /// # Returns
    /// A new `HumanStyleSolver` with candidates initialized
    pub fn new(board: &[Option<u8>]) -> Self {
        let mut solver = Self {
            board: board.to_vec(),
            candidates: CandidateGrid::new(),
            techniques_used: Vec::new(),
        };
        solver.initialize_candidates();
        solver
    }

    /// Initializes the candidate grid based on the given clues
    ///
    /// For each filled cell, removes that number as a candidate from
    /// all cells in the same row, column, and box.
    fn initialize_candidates(&mut self) {
        for index in 0..BOARD_SIZE {
            if let Some(num) = self.board[index] {
                self.place_number(index, num);
            }
        }
    }

    /// Places a number in a cell and updates all related candidates
    ///
    /// # Arguments
    /// * `index` - The cell index where to place the number
    /// * `num` - The number to place (1-9)
    fn place_number(&mut self, index: usize, num: u8) {
        let (row, col) = index_to_coords(index);

        // Set only this candidate for the cell
        self.candidates.set_only_candidate(index, num);

        // Remove this number from row, column, and box candidates
        self.eliminate_candidates_in_units(row, col, num);
    }

    /// Eliminates a number as candidate from all cells in the same units
    ///
    /// # Arguments
    /// * `row` - The row of the placed number
    /// * `col` - The column of the placed number  
    /// * `num` - The number that was placed
    fn eliminate_candidates_in_units(&mut self, row: usize, col: usize, num: u8) {
        // Remove from row and column
        for i in 0..GRID_SIZE {
            let row_idx = coords_to_index(row, i);
            let col_idx = coords_to_index(i, col);
            self.candidates.remove_candidate(row_idx, num);
            self.candidates.remove_candidate(col_idx, num);
        }

        // Remove from 3x3 box
        let box_start_row = (row / BOX_SIZE) * BOX_SIZE;
        let box_start_col = (col / BOX_SIZE) * BOX_SIZE;
        for r in box_start_row..box_start_row + BOX_SIZE {
            for c in box_start_col..box_start_col + BOX_SIZE {
                let box_idx = coords_to_index(r, c);
                self.candidates.remove_candidate(box_idx, num);
            }
        }
    }

    /// Attempts to solve the puzzle using human-style techniques
    ///
    /// Applies techniques in order of increasing difficulty until no more
    /// progress can be made or the puzzle is solved.
    ///
    /// # Returns
    /// `true` if the puzzle was solved completely
    pub fn solve_with_techniques(&mut self) -> bool {
        loop {
            let initial_board = self.board.clone();

            // Apply techniques in order of increasing complexity
            if self.apply_basic_techniques() {
                continue;
            }
            if self.apply_intermediate_techniques() {
                continue;
            }
            if self.apply_advanced_techniques() {
                continue;
            }

            // If no progress was made, we're done
            if self.board == initial_board {
                break;
            }
        }

        self.is_solved()
    }

    /// Applies basic solving techniques (naked and hidden singles)
    ///
    /// # Returns
    /// `true` if any progress was made
    pub fn apply_basic_techniques(&mut self) -> bool {
        self.find_naked_singles() || self.find_hidden_singles()
    }

    /// Reduces the puzzle by repeatedly applying basic techniques
    ///
    /// This runs naked and hidden singles until no further progress is possible.
    pub fn reduce_with_basic_techniques(&mut self) {
        while self.apply_basic_techniques() {}
    }

    /// Applies intermediate solving techniques
    ///
    /// # Returns
    /// `true` if any progress was made
    fn apply_intermediate_techniques(&mut self) -> bool {
        self.find_naked_pairs()
            || self.find_hidden_pairs()
            || self.find_box_line_reduction()
            || self.find_pointing_pairs()
    }

    /// Applies advanced solving techniques
    ///
    /// # Returns
    /// `true` if any progress was made
    fn apply_advanced_techniques(&mut self) -> bool {
        self.find_x_wing()
            || self.find_pointing_triples()
            || self.find_swordfish()
            || self.find_xy_wing()
    }

    /// Finds naked singles - cells with only one possible candidate
    ///
    /// This is the most basic technique where a cell has been narrowed
    /// down to only one possible number.
    ///
    /// # Returns
    /// `true` if any naked singles were found and filled
    fn find_naked_singles(&mut self) -> bool {
        let mut progress = false;

        for index in 0..BOARD_SIZE {
            if self.board[index].is_none() && self.candidates.candidate_count(index) == 1 {
                let candidates = self.candidates.get_candidates(index);
                if let Some(&num) = candidates.first() {
                    self.board[index] = Some(num);
                    self.place_number(index, num);
                    self.record_technique_used(SolvingTechnique::NakedSingle);
                    progress = true;
                }
            }
        }

        progress
    }

    /// Finds hidden singles - numbers that can only go in one place in a unit
    ///
    /// Checks each row, column, and box to see if any number can only
    /// be placed in one position within that unit.
    ///
    /// # Returns
    /// `true` if any hidden singles were found and filled
    fn find_hidden_singles(&mut self) -> bool {
        let mut progress = false;

        // Check all rows, columns, and boxes
        progress |= self.find_hidden_singles_in_rows();
        progress |= self.find_hidden_singles_in_columns();
        progress |= self.find_hidden_singles_in_boxes();

        if progress {
            self.record_technique_used(SolvingTechnique::HiddenSingle);
        }

        progress
    }

    /// Finds hidden singles in all rows
    fn find_hidden_singles_in_rows(&mut self) -> bool {
        let mut progress = false;
        
        for row in 0..GRID_SIZE {
            for num in 1..=9 {
                let possible_positions: Vec<usize> = (0..GRID_SIZE)
                    .map(|col| coords_to_index(row, col))
                    .filter(|&index| {
                        self.board[index].is_none() && self.candidates.has_candidate(index, num)
                    })
                    .collect();

                if possible_positions.len() == 1 {
                    let index = possible_positions[0];
                    self.board[index] = Some(num);
                    self.place_number(index, num);
                    progress = true;
                }
            }
        }
        
        progress
    }

    /// Finds hidden singles in all columns
    fn find_hidden_singles_in_columns(&mut self) -> bool {
        let mut progress = false;
        
        for col in 0..GRID_SIZE {
            for num in 1..=9 {
                let possible_positions: Vec<usize> = (0..GRID_SIZE)
                    .map(|row| coords_to_index(row, col))
                    .filter(|&index| {
                        self.board[index].is_none() && self.candidates.has_candidate(index, num)
                    })
                    .collect();

                if possible_positions.len() == 1 {
                    let index = possible_positions[0];
                    self.board[index] = Some(num);
                    self.place_number(index, num);
                    progress = true;
                }
            }
        }
        
        progress
    }

    /// Finds hidden singles in all 3x3 boxes
    fn find_hidden_singles_in_boxes(&mut self) -> bool {
        let mut progress = false;
        
        for box_row in 0..3 {
            for box_col in 0..3 {
                let start_row = box_row * BOX_SIZE;
                let start_col = box_col * BOX_SIZE;
                
                for num in 1..=9 {
                    let mut possible_positions = Vec::new();
                    
                    for r in start_row..start_row + BOX_SIZE {
                        for c in start_col..start_col + BOX_SIZE {
                            let index = coords_to_index(r, c);
                            if self.board[index].is_none() 
                                && self.candidates.has_candidate(index, num) {
                                possible_positions.push(index);
                            }
                        }
                    }

                    if possible_positions.len() == 1 {
                        let index = possible_positions[0];
                        self.board[index] = Some(num);
                        self.place_number(index, num);
                        progress = true;
                    }
                }
            }
        }
        
        progress
    }

    /// Records that a technique was used (avoiding duplicates)
    fn record_technique_used(&mut self, technique: SolvingTechnique) {
        if !self.techniques_used.contains(&technique) {
            self.techniques_used.push(technique);
        }
    }

    fn row_indices(row: usize) -> Vec<usize> {
        (0..GRID_SIZE).map(|col| coords_to_index(row, col)).collect()
    }

    fn col_indices(col: usize) -> Vec<usize> {
        (0..GRID_SIZE).map(|row| coords_to_index(row, col)).collect()
    }

    fn box_indices(box_row: usize, box_col: usize) -> Vec<usize> {
        let start_row = box_row * BOX_SIZE;
        let start_col = box_col * BOX_SIZE;
        let mut indices = Vec::with_capacity(BOX_SIZE * BOX_SIZE);
        for r in start_row..start_row + BOX_SIZE {
            for c in start_col..start_col + BOX_SIZE {
                indices.push(coords_to_index(r, c));
            }
        }
        indices
    }

    fn peers(index: usize) -> Vec<usize> {
        let (row, col) = index_to_coords(index);
        let mut peers = Vec::new();

        for i in 0..GRID_SIZE {
            let row_idx = coords_to_index(row, i);
            let col_idx = coords_to_index(i, col);
            if row_idx != index {
                peers.push(row_idx);
            }
            if col_idx != index && col_idx != row_idx {
                peers.push(col_idx);
            }
        }

        let box_start_row = (row / BOX_SIZE) * BOX_SIZE;
        let box_start_col = (col / BOX_SIZE) * BOX_SIZE;
        for r in box_start_row..box_start_row + BOX_SIZE {
            for c in box_start_col..box_start_col + BOX_SIZE {
                let box_idx = coords_to_index(r, c);
                if box_idx != index && !peers.contains(&box_idx) {
                    peers.push(box_idx);
                }
            }
        }

        peers
    }

    /// Finds naked pairs - two cells in a unit with identical candidate pairs
    fn find_naked_pairs(&mut self) -> bool {
        let mut progress = false;

        let mut apply_pairs = |indices: Vec<usize>, solver: &mut Self| {
            use std::collections::HashMap;

            let mut pairs: HashMap<(u8, u8), Vec<usize>> = HashMap::new();
            for &index in &indices {
                if solver.board[index].is_none() && solver.candidates.candidate_count(index) == 2 {
                    let candidates = solver.candidates.get_candidates(index);
                    if candidates.len() == 2 {
                        let key = (candidates[0], candidates[1]);
                        pairs.entry(key).or_default().push(index);
                    }
                }
            }

            for (pair, cells) in pairs {
                if cells.len() == 2 {
                    for &index in &indices {
                        if !cells.contains(&index) && solver.board[index].is_none() {
                            for num in [pair.0, pair.1] {
                                if solver.candidates.has_candidate(index, num) {
                                    solver.candidates.remove_candidate(index, num);
                                    progress = true;
                                }
                            }
                        }
                    }
                }
            }
        };

        for row in 0..GRID_SIZE {
            apply_pairs(Self::row_indices(row), self);
        }
        for col in 0..GRID_SIZE {
            apply_pairs(Self::col_indices(col), self);
        }
        for box_row in 0..3 {
            for box_col in 0..3 {
                apply_pairs(Self::box_indices(box_row, box_col), self);
            }
        }

        if progress {
            self.record_technique_used(SolvingTechnique::NakedPair);
        }

        progress
    }

    /// Finds hidden pairs - two numbers that can only go in two cells in a unit
    fn find_hidden_pairs(&mut self) -> bool {
        let mut progress = false;

        let mut apply_hidden_pairs = |indices: Vec<usize>, solver: &mut Self| {
            for num1 in 1..=9 {
                for num2 in (num1 + 1)..=9 {
                    let positions_num1: Vec<usize> = indices
                        .iter()
                        .copied()
                        .filter(|&index| {
                            solver.board[index].is_none()
                                && solver.candidates.has_candidate(index, num1)
                        })
                        .collect();
                    let positions_num2: Vec<usize> = indices
                        .iter()
                        .copied()
                        .filter(|&index| {
                            solver.board[index].is_none()
                                && solver.candidates.has_candidate(index, num2)
                        })
                        .collect();

                    if positions_num1.len() == 2
                        && positions_num2.len() == 2
                        && positions_num1 == positions_num2
                    {
                        for &index in &positions_num1 {
                            let existing = solver.candidates.get_candidates(index);
                            for candidate in existing {
                                if candidate != num1 && candidate != num2 {
                                    solver.candidates.remove_candidate(index, candidate);
                                    progress = true;
                                }
                            }
                        }
                    }
                }
            }
        };

        for row in 0..GRID_SIZE {
            apply_hidden_pairs(Self::row_indices(row), self);
        }
        for col in 0..GRID_SIZE {
            apply_hidden_pairs(Self::col_indices(col), self);
        }
        for box_row in 0..3 {
            for box_col in 0..3 {
                apply_hidden_pairs(Self::box_indices(box_row, box_col), self);
            }
        }

        if progress {
            self.record_technique_used(SolvingTechnique::HiddenPair);
        }

        progress
    }

    /// Finds box-line reduction patterns
    fn find_box_line_reduction(&mut self) -> bool {
        let mut progress = false;

        for row in 0..GRID_SIZE {
            for num in 1..=9 {
                let positions: Vec<usize> = Self::row_indices(row)
                    .into_iter()
                    .filter(|&index| {
                        self.board[index].is_none() && self.candidates.has_candidate(index, num)
                    })
                    .collect();

                if positions.len() >= 2 {
                    let box_col = positions
                        .iter()
                        .map(|&index| index_to_coords(index).1 / BOX_SIZE)
                        .collect::<std::collections::HashSet<_>>();
                    if box_col.len() == 1 {
                        let target_box_col = *box_col.iter().next().unwrap();
                        let box_row = row / BOX_SIZE;
                        for index in Self::box_indices(box_row, target_box_col) {
                            let (r, _) = index_to_coords(index);
                            if r != row
                                && self.board[index].is_none()
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

        for col in 0..GRID_SIZE {
            for num in 1..=9 {
                let positions: Vec<usize> = Self::col_indices(col)
                    .into_iter()
                    .filter(|&index| {
                        self.board[index].is_none() && self.candidates.has_candidate(index, num)
                    })
                    .collect();

                if positions.len() >= 2 {
                    let box_row = positions
                        .iter()
                        .map(|&index| index_to_coords(index).0 / BOX_SIZE)
                        .collect::<std::collections::HashSet<_>>();
                    if box_row.len() == 1 {
                        let target_box_row = *box_row.iter().next().unwrap();
                        let box_col = col / BOX_SIZE;
                        for index in Self::box_indices(target_box_row, box_col) {
                            let (_, c) = index_to_coords(index);
                            if c != col
                                && self.board[index].is_none()
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

        if progress {
            self.record_technique_used(SolvingTechnique::BoxLineReduction);
        }

        progress
    }

    /// Finds pointing pairs patterns
    fn find_pointing_pairs(&mut self) -> bool {
        let mut progress = false;

        for box_row in 0..3 {
            for box_col in 0..3 {
                let indices = Self::box_indices(box_row, box_col);
                for num in 1..=9 {
                    let positions: Vec<usize> = indices
                        .iter()
                        .copied()
                        .filter(|&index| {
                            self.board[index].is_none() && self.candidates.has_candidate(index, num)
                        })
                        .collect();

                    if positions.len() >= 2 {
                        let rows: std::collections::HashSet<usize> = positions
                            .iter()
                            .map(|&index| index_to_coords(index).0)
                            .collect();
                        let cols: std::collections::HashSet<usize> = positions
                            .iter()
                            .map(|&index| index_to_coords(index).1)
                            .collect();

                        if rows.len() == 1 {
                            let row = *rows.iter().next().unwrap();
                            for index in Self::row_indices(row) {
                                let (_, col) = index_to_coords(index);
                                if col / BOX_SIZE != box_col
                                    && self.board[index].is_none()
                                    && self.candidates.has_candidate(index, num)
                                {
                                    self.candidates.remove_candidate(index, num);
                                    progress = true;
                                }
                            }
                        }

                        if cols.len() == 1 {
                            let col = *cols.iter().next().unwrap();
                            for index in Self::col_indices(col) {
                                let (row, _) = index_to_coords(index);
                                if row / BOX_SIZE != box_row
                                    && self.board[index].is_none()
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

        if progress {
            self.record_technique_used(SolvingTechnique::PointingPairs);
        }

        progress
    }

    /// Finds X-Wing patterns
    fn find_x_wing(&mut self) -> bool {
        let mut progress = false;

        for num in 1..=9 {
            let mut row_candidates = Vec::new();
            for row in 0..GRID_SIZE {
                let cols: Vec<usize> = Self::row_indices(row)
                    .into_iter()
                    .filter(|&index| {
                        self.board[index].is_none() && self.candidates.has_candidate(index, num)
                    })
                    .map(|index| index_to_coords(index).1)
                    .collect();
                if cols.len() == 2 {
                    row_candidates.push((row, cols));
                }
            }

            for i in 0..row_candidates.len() {
                for j in (i + 1)..row_candidates.len() {
                    if row_candidates[i].1 == row_candidates[j].1 {
                        let cols = &row_candidates[i].1;
                        for row in 0..GRID_SIZE {
                            if row != row_candidates[i].0 && row != row_candidates[j].0 {
                                for &col in cols {
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

            let mut col_candidates = Vec::new();
            for col in 0..GRID_SIZE {
                let rows: Vec<usize> = Self::col_indices(col)
                    .into_iter()
                    .filter(|&index| {
                        self.board[index].is_none() && self.candidates.has_candidate(index, num)
                    })
                    .map(|index| index_to_coords(index).0)
                    .collect();
                if rows.len() == 2 {
                    col_candidates.push((col, rows));
                }
            }

            for i in 0..col_candidates.len() {
                for j in (i + 1)..col_candidates.len() {
                    if col_candidates[i].1 == col_candidates[j].1 {
                        let rows = &col_candidates[i].1;
                        for col in 0..GRID_SIZE {
                            if col != col_candidates[i].0 && col != col_candidates[j].0 {
                                for &row in rows {
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

        if progress {
            self.record_technique_used(SolvingTechnique::XWing);
        }

        progress
    }

    /// Finds pointing triples patterns
    fn find_pointing_triples(&mut self) -> bool {
        // Implementation would go here
        false
    }

    /// Finds Swordfish patterns
    fn find_swordfish(&mut self) -> bool {
        let mut progress = false;

        for num in 1..=9 {
            let mut row_candidates = Vec::new();
            for row in 0..GRID_SIZE {
                let cols: Vec<usize> = Self::row_indices(row)
                    .into_iter()
                    .filter(|&index| {
                        self.board[index].is_none() && self.candidates.has_candidate(index, num)
                    })
                    .map(|index| index_to_coords(index).1)
                    .collect();
                if (2..=3).contains(&cols.len()) {
                    row_candidates.push((row, cols));
                }
            }

            for i in 0..row_candidates.len() {
                for j in (i + 1)..row_candidates.len() {
                    for k in (j + 1)..row_candidates.len() {
                        let mut cols = row_candidates[i].1.clone();
                        cols.extend(&row_candidates[j].1);
                        cols.extend(&row_candidates[k].1);
                        cols.sort_unstable();
                        cols.dedup();
                        if cols.len() == 3 {
                            for row in 0..GRID_SIZE {
                                if row != row_candidates[i].0
                                    && row != row_candidates[j].0
                                    && row != row_candidates[k].0
                                {
                                    for &col in &cols {
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

            let mut col_candidates = Vec::new();
            for col in 0..GRID_SIZE {
                let rows: Vec<usize> = Self::col_indices(col)
                    .into_iter()
                    .filter(|&index| {
                        self.board[index].is_none() && self.candidates.has_candidate(index, num)
                    })
                    .map(|index| index_to_coords(index).0)
                    .collect();
                if (2..=3).contains(&rows.len()) {
                    col_candidates.push((col, rows));
                }
            }

            for i in 0..col_candidates.len() {
                for j in (i + 1)..col_candidates.len() {
                    for k in (j + 1)..col_candidates.len() {
                        let mut rows = col_candidates[i].1.clone();
                        rows.extend(&col_candidates[j].1);
                        rows.extend(&col_candidates[k].1);
                        rows.sort_unstable();
                        rows.dedup();
                        if rows.len() == 3 {
                            for col in 0..GRID_SIZE {
                                if col != col_candidates[i].0
                                    && col != col_candidates[j].0
                                    && col != col_candidates[k].0
                                {
                                    for &row in &rows {
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
        }

        if progress {
            self.record_technique_used(SolvingTechnique::Swordfish);
        }

        progress
    }

    /// Finds XY-Wing patterns
    fn find_xy_wing(&mut self) -> bool {
        let mut progress = false;

        for pivot in 0..BOARD_SIZE {
            if self.board[pivot].is_some() || self.candidates.candidate_count(pivot) != 2 {
                continue;
            }
            let pivot_candidates = self.candidates.get_candidates(pivot);
            let pivot_a = pivot_candidates[0];
            let pivot_b = pivot_candidates[1];

            let pivot_peers = Self::peers(pivot);
            for &wing1 in &pivot_peers {
                if self.board[wing1].is_some() || self.candidates.candidate_count(wing1) != 2 {
                    continue;
                }
                let wing1_candidates = self.candidates.get_candidates(wing1);
                if !wing1_candidates.contains(&pivot_a) || wing1_candidates.contains(&pivot_b) {
                    continue;
                }
                let wing1_z = *wing1_candidates
                    .iter()
                    .find(|&&num| num != pivot_a)
                    .unwrap();

                for &wing2 in &pivot_peers {
                    if wing2 == wing1
                        || self.board[wing2].is_some()
                        || self.candidates.candidate_count(wing2) != 2
                    {
                        continue;
                    }
                    let wing2_candidates = self.candidates.get_candidates(wing2);
                    if !wing2_candidates.contains(&pivot_b)
                        || wing2_candidates.contains(&pivot_a)
                    {
                        continue;
                    }
                    let wing2_z = *wing2_candidates
                        .iter()
                        .find(|&&num| num != pivot_b)
                        .unwrap();

                    if wing1_z != wing2_z {
                        continue;
                    }

                    let wing1_peers = Self::peers(wing1);
                    let wing2_peers = Self::peers(wing2);
                    for index in wing1_peers.iter().filter(|idx| wing2_peers.contains(idx)) {
                        if self.board[*index].is_none()
                            && self.candidates.has_candidate(*index, wing1_z)
                        {
                            self.candidates.remove_candidate(*index, wing1_z);
                            progress = true;
                        }
                    }
                }
            }
        }

        if progress {
            self.record_technique_used(SolvingTechnique::XYWing);
        }

        progress
    }

    /// Checks if the puzzle is completely solved
    ///
    /// # Returns
    /// `true` if all cells are filled
    pub fn is_solved(&self) -> bool {
        self.board.iter().all(|&cell| cell.is_some())
    }

    /// Calculates the branching factor (average candidates per empty cell)
    ///
    /// This metric indicates puzzle complexity - higher values mean more
    /// choices and potentially more difficult puzzles.
    ///
    /// # Returns
    /// The average number of candidates per empty cell
    pub fn calculate_branching_factor(&self) -> f64 {
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

    /// Gets the hardest technique that was used during solving
    ///
    /// # Returns
    /// The most advanced technique from the solving process
    pub fn get_hardest_technique_used(&self) -> SolvingTechnique {
        self.techniques_used
            .iter()
            .max()
            .cloned()
            .unwrap_or(SolvingTechnique::NakedSingle)
    }

    /// Gets all techniques that were used during solving
    ///
    /// # Returns
    /// Reference to the list of techniques used
    pub fn get_techniques_used(&self) -> &[SolvingTechnique] {
        &self.techniques_used
    }

    /// Gets the current board state
    ///
    /// # Returns
    /// Reference to the current board
    pub fn get_board(&self) -> &[Option<u8>] {
        &self.board
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const EASY_BRANCHING_FIXTURE: [Option<u8>; 81] = [
        // Row 1
        None,
        None,
        Some(5),
        None,
        Some(7),
        None,
        Some(1),
        Some(6),
        Some(3),
        // Row 2
        None,
        None,
        Some(6),
        Some(1),
        Some(5),
        Some(4),
        None,
        None,
        Some(2),
        // Row 3
        Some(9),
        Some(1),
        None,
        Some(6),
        None,
        None,
        Some(7),
        None,
        Some(4),
        // Row 4
        None,
        None,
        Some(4),
        Some(3),
        None,
        Some(7),
        None,
        None,
        Some(6),
        // Row 5
        None,
        Some(9),
        None,
        Some(4),
        None,
        Some(6),
        None,
        Some(7),
        None,
        // Row 6
        Some(6),
        None,
        None,
        Some(2),
        None,
        Some(5),
        Some(4),
        None,
        None,
        // Row 7
        Some(8),
        None,
        Some(9),
        None,
        None,
        Some(1),
        None,
        Some(2),
        Some(7),
        // Row 8
        Some(1),
        None,
        None,
        Some(9),
        Some(3),
        Some(2),
        Some(8),
        None,
        None,
        // Row 9
        Some(3),
        Some(5),
        Some(2),
        None,
        Some(4),
        None,
        Some(6),
        None,
        None,
    ];

    fn set_candidates(solver: &mut HumanStyleSolver, row: usize, col: usize, candidates: &[u8]) {
        let index = coords_to_index(row, col);
        for num in 1..=9 {
            if !candidates.contains(&num) {
                solver.candidates.remove_candidate(index, num);
            }
        }
    }

    fn retain_candidate_only_at(
        solver: &mut HumanStyleSolver,
        num: u8,
        allowed_indices: &[usize],
    ) {
        for index in 0..BOARD_SIZE {
            if !allowed_indices.contains(&index) {
                solver.candidates.remove_candidate(index, num);
            }
        }
    }

    fn remove_candidate_from_row_except(
        solver: &mut HumanStyleSolver,
        row: usize,
        num: u8,
        allowed_cols: &[usize],
    ) {
        for col in 0..GRID_SIZE {
            if !allowed_cols.contains(&col) {
                solver.candidates.remove_candidate(coords_to_index(row, col), num);
            }
        }
    }

    #[test]
    fn test_solver_creation() {
        let board = vec![None; BOARD_SIZE];
        let solver = HumanStyleSolver::new(&board);
        assert_eq!(solver.get_board().len(), BOARD_SIZE);
        assert!(!solver.is_solved());
    }

    #[test]
    fn test_branching_factor_empty_board() {
        let board = vec![None; BOARD_SIZE];
        let solver = HumanStyleSolver::new(&board);
        // Empty board should have high branching factor
        assert!(solver.calculate_branching_factor() > 5.0);
    }

    #[test]
    fn test_branching_factor_full_board() {
        let board = vec![Some(1); BOARD_SIZE];
        let solver = HumanStyleSolver::new(&board);
        // Full board should have low branching factor
        assert_eq!(solver.calculate_branching_factor(), 1.0);
    }

    #[test]
    fn test_branching_factor_reduction_with_basic_techniques() {
        let mut solver = HumanStyleSolver::new(&EASY_BRANCHING_FIXTURE);
        let before = solver.calculate_branching_factor();

        solver.reduce_with_basic_techniques();
        let after = solver.calculate_branching_factor();

        println!("Branching factor before: {:.2}, after: {:.2}", before, after);

        assert!(
            after <= before,
            "Expected branching factor to reduce or stay the same (before {:.2}, after {:.2})",
            before,
            after
        );
        assert!(
            after >= 1.6 && after <= 2.2,
            "Expected reduced branching factor in easy-tier range (1.6-2.2), got {:.2}",
            after
        );
    }

    #[test]
    fn test_naked_pairs_technique() {
        let board = vec![None; BOARD_SIZE];
        let mut solver = HumanStyleSolver::new(&board);

        set_candidates(&mut solver, 2, 0, &[1, 2]);
        set_candidates(&mut solver, 2, 1, &[1, 2]);
        set_candidates(&mut solver, 2, 2, &[1, 2, 3]);

        assert!(solver.find_naked_pairs());

        let target = coords_to_index(2, 2);
        assert_eq!(solver.candidates.get_candidates(target), vec![3]);
        assert_eq!(solver.get_hardest_technique_used(), SolvingTechnique::NakedPair);
    }

    #[test]
    fn test_hidden_pairs_technique() {
        let board = vec![None; BOARD_SIZE];
        let mut solver = HumanStyleSolver::new(&board);

        remove_candidate_from_row_except(&mut solver, 1, 3, &[0, 1]);
        remove_candidate_from_row_except(&mut solver, 1, 4, &[0, 1]);
        set_candidates(&mut solver, 1, 0, &[3, 4, 5]);
        set_candidates(&mut solver, 1, 1, &[3, 4, 6]);

        assert!(solver.find_hidden_pairs());

        let first = coords_to_index(1, 0);
        let second = coords_to_index(1, 1);
        assert_eq!(solver.candidates.get_candidates(first), vec![3, 4]);
        assert_eq!(solver.candidates.get_candidates(second), vec![3, 4]);
        assert_eq!(solver.get_hardest_technique_used(), SolvingTechnique::HiddenPair);
    }

    #[test]
    fn test_box_line_reduction_technique() {
        let board = vec![None; BOARD_SIZE];
        let mut solver = HumanStyleSolver::new(&board);

        remove_candidate_from_row_except(&mut solver, 0, 5, &[0, 1]);
        let target = coords_to_index(1, 2);
        assert!(solver.candidates.has_candidate(target, 5));

        assert!(solver.find_box_line_reduction());

        assert!(!solver.candidates.has_candidate(target, 5));
        assert_eq!(
            solver.get_hardest_technique_used(),
            SolvingTechnique::BoxLineReduction
        );
    }

    #[test]
    fn test_pointing_pairs_technique() {
        let board = vec![None; BOARD_SIZE];
        let mut solver = HumanStyleSolver::new(&board);

        set_candidates(&mut solver, 0, 0, &[6, 7]);
        set_candidates(&mut solver, 0, 1, &[6, 8]);
        for row in 0..BOX_SIZE {
            for col in 0..BOX_SIZE {
                if !(row == 0 && (col == 0 || col == 1)) {
                    solver
                        .candidates
                        .remove_candidate(coords_to_index(row, col), 6);
                }
            }
        }
        let target = coords_to_index(0, 4);
        assert!(solver.candidates.has_candidate(target, 6));

        assert!(solver.find_pointing_pairs());

        assert!(!solver.candidates.has_candidate(target, 6));
        assert_eq!(
            solver.get_hardest_technique_used(),
            SolvingTechnique::PointingPairs
        );
    }

    #[test]
    fn test_x_wing_technique() {
        let board = vec![None; BOARD_SIZE];
        let mut solver = HumanStyleSolver::new(&board);

        let allowed = vec![
            coords_to_index(0, 1),
            coords_to_index(0, 5),
            coords_to_index(3, 1),
            coords_to_index(3, 5),
            coords_to_index(6, 1),
        ];
        retain_candidate_only_at(&mut solver, 7, &allowed);

        let target = coords_to_index(6, 1);
        assert!(solver.candidates.has_candidate(target, 7));

        assert!(solver.find_x_wing());

        assert!(!solver.candidates.has_candidate(target, 7));
        assert_eq!(solver.get_hardest_technique_used(), SolvingTechnique::XWing);
    }

    #[test]
    fn test_swordfish_technique() {
        let board = vec![None; BOARD_SIZE];
        let mut solver = HumanStyleSolver::new(&board);

        let allowed = vec![
            coords_to_index(0, 0),
            coords_to_index(0, 3),
            coords_to_index(0, 6),
            coords_to_index(1, 0),
            coords_to_index(1, 3),
            coords_to_index(1, 6),
            coords_to_index(2, 0),
            coords_to_index(2, 3),
            coords_to_index(2, 6),
            coords_to_index(5, 3),
        ];
        retain_candidate_only_at(&mut solver, 8, &allowed);

        let target = coords_to_index(5, 3);
        assert!(solver.candidates.has_candidate(target, 8));

        assert!(solver.find_swordfish());

        assert!(!solver.candidates.has_candidate(target, 8));
        assert_eq!(solver.get_hardest_technique_used(), SolvingTechnique::Swordfish);
    }

    #[test]
    fn test_xy_wing_technique() {
        let board = vec![None; BOARD_SIZE];
        let mut solver = HumanStyleSolver::new(&board);

        set_candidates(&mut solver, 0, 0, &[1, 2]);
        set_candidates(&mut solver, 0, 4, &[1, 3]);
        set_candidates(&mut solver, 4, 0, &[2, 3]);
        set_candidates(&mut solver, 4, 4, &[3, 9]);

        let target = coords_to_index(4, 4);
        assert!(solver.candidates.has_candidate(target, 3));

        assert!(solver.find_xy_wing());

        assert!(!solver.candidates.has_candidate(target, 3));
        assert_eq!(solver.get_hardest_technique_used(), SolvingTechnique::XYWing);
    }
}
