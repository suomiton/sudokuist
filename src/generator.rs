//! Sudoku puzzle generator with difficulty control
//!
//! This module generates valid Sudoku puzzles with specific difficulty targets
//! by using a combination of solution generation and selective clue removal.

use crate::difficulty::analyze_difficulty;
use crate::types::{DifficultyAnalysis, DifficultyLevel, SolvingTechnique, BOARD_SIZE};
use crate::validator::has_unique_solution;
use rand::seq::SliceRandom;
use rand::{thread_rng, Rng};

#[cfg(target_arch = "wasm32")]
use web_sys;

/// Configuration for puzzle generation
#[derive(Debug, Clone)]
pub struct GeneratorConfig {
    pub target_difficulty: DifficultyLevel,
    pub max_attempts: u32,
    pub min_clues: usize,
    pub max_clues: usize,
    pub prefer_symmetry: bool,
}

impl Default for GeneratorConfig {
    fn default() -> Self {
        Self {
            target_difficulty: DifficultyLevel::Medium,
            max_attempts: 1000,
            min_clues: 17, // Theoretical minimum for unique solutions
            max_clues: 35,
            prefer_symmetry: true,
        }
    }
}

/// A Sudoku puzzle generator that creates puzzles with specific difficulty levels
pub struct PuzzleGenerator {
    config: GeneratorConfig,
}

impl PuzzleGenerator {
    /// Creates a new puzzle generator with the given configuration
    pub fn new(config: GeneratorConfig) -> Self {
        Self { config }
    }

    /// Creates a generator with default settings for a specific difficulty
    pub fn with_difficulty(difficulty: DifficultyLevel) -> Self {
        let mut config = GeneratorConfig::default();
        config.target_difficulty = difficulty;

        // Adjust clue range based on difficulty
        match difficulty {
            DifficultyLevel::Easy => {
                config.min_clues = 35;
                config.max_clues = 45; // a bit roomier for very easy
            }
            DifficultyLevel::Medium => {
                config.min_clues = 28;
                config.max_clues = 35;
            }
            DifficultyLevel::Hard => {
                config.min_clues = 22;
                config.max_clues = 28;
            }
            DifficultyLevel::Expert => {
                config.min_clues = 17;
                config.max_clues = 25;
            }
        }

        Self::new(config)
    }

    /// Generates a new puzzle with the configured difficulty
    pub fn generate(&self) -> Option<Vec<Option<u8>>> {
        for attempt in 0..self.config.max_attempts {
            if let Some(puzzle) = self.generate_attempt() {
                if self.validate_puzzle(&puzzle) {
                    return Some(puzzle);
                }
            }

            // Log progress on WASM builds every 100 attempts
            #[cfg(target_arch = "wasm32")]
            if attempt % 100 == 0 && attempt > 0 {
                web_sys::console::log_1(&format!("Generation attempt {}", attempt).into());
            }
        }

        None
    }

    /// Attempts to generate a single puzzle
    fn generate_attempt(&self) -> Option<Vec<Option<u8>>> {
        let solution = self.generate_complete_solution()?;
        self.create_puzzle_from_solution(&solution)
    }

    /// Generates a complete, valid Sudoku solution
    ///
    /// Uses a fast bit‑mask back‑tracking algorithm.
    fn generate_complete_solution(&self) -> Option<Vec<Option<u8>>> {
        let mut board = [0u8; BOARD_SIZE]; // 0 represents empty
        let mut row_mask = [0u16; 9];
        let mut col_mask = [0u16; 9];
        let mut box_mask = [0u16; 9];
        let mut rng = thread_rng();

        if self.fill_board_fast(
            &mut board,
            &mut row_mask,
            &mut col_mask,
            &mut box_mask,
            0,
            &mut rng,
        ) {
            // Convert to external Vec<Option<u8>> representation
            Some(
                board
                    .iter()
                    .map(|&n| if n == 0 { None } else { Some(n) })
                    .collect(),
            )
        } else {
            None
        }
    }

    /// Recursive fast solver using bit masks and randomised value order
    fn fill_board_fast(
        &self,
        board: &mut [u8; BOARD_SIZE],
        row_mask: &mut [u16; 9],
        col_mask: &mut [u16; 9],
        box_mask: &mut [u16; 9],
        idx: usize,
        rng: &mut impl Rng,
    ) -> bool {
        if idx == BOARD_SIZE {
            return true;
        }

        let r = idx / 9;
        let c = idx % 9;
        let b = (r / 3) * 3 + (c / 3);

        if board[idx] != 0 {
            return self.fill_board_fast(board, row_mask, col_mask, box_mask, idx + 1, rng);
        }

        let mut digits: [u8; 9] = [1, 2, 3, 4, 5, 6, 7, 8, 9];
        digits.shuffle(rng);

        for &d in &digits {
            let bit = 1 << d;
            if row_mask[r] & bit == 0 && col_mask[c] & bit == 0 && box_mask[b] & bit == 0 {
                board[idx] = d;
                row_mask[r] |= bit;
                col_mask[c] |= bit;
                box_mask[b] |= bit;

                if self.fill_board_fast(board, row_mask, col_mask, box_mask, idx + 1, rng) {
                    return true;
                }

                board[idx] = 0;
                row_mask[r] ^= bit;
                col_mask[c] ^= bit;
                box_mask[b] ^= bit;
            }
        }

        false
    }

    /// Creates a puzzle by strategically removing clues from a complete solution
    fn create_puzzle_from_solution(&self, solution: &[Option<u8>]) -> Option<Vec<Option<u8>>> {
        let mut puzzle = solution.to_vec();
        let removal_order = self.get_removal_order();
        let mut removals_since_last_uniqueness = 0;

        for &idx in &removal_order {
            let original = puzzle[idx];
            puzzle[idx] = None;

            // Fast clue‑count guard
            if !self.meets_clue_constraints(&puzzle) {
                puzzle[idx] = original;
                continue;
            }

            // Light throttling: expensive uniqueness test only every 3 removals
            let need_uniqueness_now = removals_since_last_uniqueness >= 3
                || puzzle.iter().filter(|c| c.is_some()).count() <= self.config.min_clues + 2;

            if need_uniqueness_now && !has_unique_solution(&puzzle) {
                puzzle[idx] = original;
                removals_since_last_uniqueness = 0;
                continue;
            }

            removals_since_last_uniqueness = if need_uniqueness_now {
                0
            } else {
                removals_since_last_uniqueness + 1
            };

            // Difficulty check – early exit as soon as requirements met
            if self.meets_clue_constraints(&puzzle) {
                let analysis = analyze_difficulty(&puzzle);
                if self.difficulty_matches_target(&analysis) {
                    return Some(puzzle);
                }
            }

            // Emergency stop at min clues
            if puzzle.iter().filter(|c| c.is_some()).count() <= self.config.min_clues {
                break;
            }
        }

        // Final validation (covers edge cases where early exit didn't happen)
        if self.validate_puzzle(&puzzle) {
            Some(puzzle)
        } else {
            None
        }
    }

    /// Generates the order in which to attempt clue removal
    fn get_removal_order(&self) -> Vec<usize> {
        let mut rng = thread_rng();
        let mut indices: Vec<usize> = (0..BOARD_SIZE).collect();

        if self.config.prefer_symmetry {
            // Pair indices with their 180‑degree counterparts
            let mut pairs = Vec::<(usize, usize)>::new();
            let mut visited = vec![false; BOARD_SIZE];

            for i in 0..BOARD_SIZE {
                if !visited[i] {
                    let sym = self.get_symmetric_index(i);
                    if sym != i && !visited[sym] {
                        pairs.push((i, sym));
                        visited[i] = true;
                        visited[sym] = true;
                    } else {
                        pairs.push((i, i)); // center cell or already paired
                        visited[i] = true;
                    }
                }
            }

            pairs.shuffle(&mut rng);
            indices = pairs
                .into_iter()
                .flat_map(|(a, b)| if a == b { vec![a] } else { vec![a, b] })
                .collect();
        } else {
            indices.shuffle(&mut rng);
        }

        indices
    }

    /// Gets the symmetric counterpart of a cell index (180‑degree rotation)
    fn get_symmetric_index(&self, index: usize) -> usize {
        let row = index / 9;
        let col = index % 9;
        (8 - row) * 9 + (8 - col)
    }

    /// Checks if the puzzle meets the clue count constraints
    fn meets_clue_constraints(&self, puzzle: &[Option<u8>]) -> bool {
        let clues = puzzle.iter().filter(|c| c.is_some()).count();
        clues >= self.config.min_clues && clues <= self.config.max_clues
    }

    /// Validates that a generated puzzle meets all requirements
    fn validate_puzzle(&self, puzzle: &[Option<u8>]) -> bool {
        if !has_unique_solution(puzzle) || !self.meets_clue_constraints(puzzle) {
            return false;
        }
        let analysis = analyze_difficulty(puzzle);
        self.difficulty_matches_target(&analysis)
    }

    /// Checks if the analyzed difficulty matches the target
    fn difficulty_matches_target(&self, analysis: &DifficultyAnalysis) -> bool {
        match self.config.target_difficulty {
            DifficultyLevel::Easy => {
                analysis.hardest_technique <= SolvingTechnique::HiddenSingle
                    && analysis.technique_diversity <= 2
            }
            DifficultyLevel::Medium => {
                analysis.hardest_technique <= SolvingTechnique::BoxLineReduction
                    && analysis.technique_diversity <= 4
            }
            DifficultyLevel::Hard => {
                analysis.hardest_technique <= SolvingTechnique::XWing
                    && analysis.technique_diversity <= 6
            }
            DifficultyLevel::Expert => {
                analysis.hardest_technique >= SolvingTechnique::Swordfish
                    || analysis.technique_diversity >= 6
            }
        }
    }

    // -----------------------------------------------------------------
    // Legacy helpers kept for signature compatibility
    // -----------------------------------------------------------------

    /// Recursively fills the board using randomized backtracking (legacy)
    ///
    /// Retained so external callers/tests that reflect on this symbol compile.
    #[allow(dead_code)]
    fn fill_board_randomly(&self, _board: &mut Vec<Option<u8>>, _index: usize) -> bool {
        unreachable!("Replaced by faster bit‑mask solver; kept for API stability")
    }

    /// Checks if placing a number at a position is valid (legacy)
    #[allow(dead_code)]
    fn is_valid_placement(&self, board: &[Option<u8>], index: usize, num: u8) -> bool {
        let row = index / 9;
        let col = index % 9;

        // Row
        if (0..9).any(|c| board[row * 9 + c] == Some(num)) {
            return false;
        }
        // Column
        if (0..9).any(|r| board[r * 9 + col] == Some(num)) {
            return false;
        }
        // Box
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
}

/// Convenience helper: generate one puzzle
pub fn generate_puzzle(difficulty: DifficultyLevel) -> Option<Vec<Option<u8>>> {
    let generator = PuzzleGenerator::with_difficulty(difficulty);
    generator.generate()
}

/// Convenience helper: generate multiple puzzles
pub fn generate_multiple_puzzles(
    difficulty: DifficultyLevel,
    count: usize,
) -> Vec<Vec<Option<u8>>> {
    let generator = PuzzleGenerator::with_difficulty(difficulty);
    let mut puzzles = Vec::with_capacity(count);

    for _ in 0..count {
        if let Some(puz) = generator.generate() {
            puzzles.push(puz);
        }
    }

    puzzles
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generator_creation() {
        let g = PuzzleGenerator::with_difficulty(DifficultyLevel::Easy);
        assert_eq!(g.config.target_difficulty, DifficultyLevel::Easy);
    }

    #[test]
    fn test_symmetric_index_calculation() {
        let g = PuzzleGenerator::with_difficulty(DifficultyLevel::Medium);
        assert_eq!(g.get_symmetric_index(0), 80);
        assert_eq!(g.get_symmetric_index(8), 72);
        assert_eq!(g.get_symmetric_index(72), 8);
        assert_eq!(g.get_symmetric_index(80), 0);
        assert_eq!(g.get_symmetric_index(40), 40);
    }

    #[test]
    fn test_clue_constraints() {
        let g = PuzzleGenerator::with_difficulty(DifficultyLevel::Easy);
        let too_few = vec![None; BOARD_SIZE];
        let good: Vec<Option<u8>> = (0..40)
            .map(|_| Some(1))
            .chain(std::iter::repeat(None).take(BOARD_SIZE - 40))
            .collect();

        assert!(!g.meets_clue_constraints(&too_few));
        assert!(g.meets_clue_constraints(&good));
    }
}
