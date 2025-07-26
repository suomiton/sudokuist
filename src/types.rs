//! Type definitions and constants for the Sudoku engine
//!
//! This module contains all the core type definitions, enums, and constants
//! used throughout the Sudoku application.

use serde::{Deserialize, Serialize};

/// Sudoku board dimensions
pub const BOARD_SIZE: usize = 81;
pub const GRID_SIZE: usize = 9;
pub const BOX_SIZE: usize = 3;

/// Result of board validation containing invalid cell indices and completion status
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ValidationResult {
    /// Indices of cells that contain invalid values (conflicts with Sudoku rules)
    pub invalid_indices: Vec<usize>,
    /// Whether the board is completely filled and valid
    pub is_complete: bool,
}

/// Difficulty levels for Sudoku puzzles
///
/// Represents the overall difficulty rating of a puzzle based on the
/// techniques required to solve it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum DifficultyLevel {
    /// Requires only basic techniques (naked/hidden singles)
    Easy,
    /// Requires intermediate techniques (pairs, box-line reduction)
    Medium,
    /// Requires advanced techniques (X-Wing, complex patterns)
    Hard,
    /// Requires expert techniques (Swordfish, chains, coloring)
    Expert,
}

/// Enumeration of Sudoku solving techniques ordered by difficulty
///
/// Each technique represents a logical method that humans use to solve Sudoku puzzles.
/// The ordering (via PartialOrd) represents increasing difficulty levels.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum SolvingTechnique {
    /// Basic technique: cell has only one possible candidate
    NakedSingle,
    /// Basic technique: number can only go in one place in a unit (row/column/box)
    HiddenSingle,
    /// Intermediate: two cells in a unit have identical pair of candidates
    NakedPair,
    /// Intermediate: two numbers appear in only two cells in a unit
    HiddenPair,
    /// Intermediate: candidates in a box are confined to one row/column
    BoxLineReduction,
    /// Intermediate: candidates in a row/column are confined to one box
    PointingPairs,
    /// Advanced: four cells form a rectangle pattern for elimination
    XWing,
    /// Advanced: three cells in a row/column point to same box
    PointingTriples,
    /// Expert: six cells form a complex elimination pattern
    Swordfish,
    /// Expert: color-based candidate elimination
    Coloring,
    /// Expert: three bi-value cells form a wing pattern
    XYWing,
    /// Expert: chain of bi-value cells
    XYChain,
    /// Expert: chain of strong/weak links
    ForcingChain,
    /// Last resort: guessing and backtracking
    TrialAndError,
}

/// Comprehensive analysis of a puzzle's difficulty characteristics
#[derive(Debug, Clone)]
pub struct DifficultyAnalysis {
    /// Overall difficulty level
    pub level: DifficultyLevel,
    /// The most advanced technique needed to solve the puzzle
    pub hardest_technique: SolvingTechnique,
    /// Number of different techniques required (diversity metric)
    pub technique_diversity: usize,
    /// Average number of candidates per empty cell (complexity metric)
    pub branching_factor: f64,
}

/// Cell candidates tracking using bit flags for efficient storage and operations
///
/// Each cell's candidates are stored as a 16-bit integer where each bit
/// represents whether numbers 1-9 are possible candidates.
#[derive(Clone, Debug)]
pub struct CandidateGrid {
    /// Bit flags for candidates 1-9 for each of the 81 cells
    /// Bit 0 = candidate 1, bit 1 = candidate 2, etc.
    candidates: [u16; BOARD_SIZE],
}

impl CandidateGrid {
    /// Creates a new candidate grid with all possibilities initially available
    ///
    /// # Returns
    /// A new `CandidateGrid` where every cell can contain any number 1-9
    pub fn new() -> Self {
        Self {
            // 0b111111111 = all 9 bits set for candidates 1-9
            candidates: [0b111111111; BOARD_SIZE],
        }
    }

    /// Checks if a specific number is a candidate for a given cell
    ///
    /// # Arguments
    /// * `index` - The cell index (0-80)
    /// * `num` - The number to check (1-9)
    ///
    /// # Returns
    /// `true` if the number is a possible candidate for the cell
    pub fn has_candidate(&self, index: usize, num: u8) -> bool {
        self.candidates[index] & (1 << (num - 1)) != 0
    }

    /// Removes a number as a candidate from a specific cell
    ///
    /// # Arguments
    /// * `index` - The cell index (0-80)
    /// * `num` - The number to remove as candidate (1-9)
    pub fn remove_candidate(&mut self, index: usize, num: u8) {
        self.candidates[index] &= !(1 << (num - 1));
    }

    /// Sets a cell to have only one specific candidate
    ///
    /// # Arguments
    /// * `index` - The cell index (0-80)
    /// * `num` - The only candidate number (1-9)
    pub fn set_only_candidate(&mut self, index: usize, num: u8) {
        self.candidates[index] = 1 << (num - 1);
    }

    /// Gets all candidate numbers for a specific cell
    ///
    /// # Arguments
    /// * `index` - The cell index (0-80)
    ///
    /// # Returns
    /// A vector containing all possible candidate numbers for the cell
    pub fn get_candidates(&self, index: usize) -> Vec<u8> {
        let mut result = Vec::new();
        for num in 1..=9 {
            if self.has_candidate(index, num) {
                result.push(num);
            }
        }
        result
    }

    /// Counts the number of candidates for a specific cell
    ///
    /// # Arguments
    /// * `index` - The cell index (0-80)
    ///
    /// # Returns
    /// The number of possible candidates for the cell
    pub fn candidate_count(&self, index: usize) -> usize {
        self.candidates[index].count_ones() as usize
    }
}

impl Default for CandidateGrid {
    fn default() -> Self {
        Self::new()
    }
}
