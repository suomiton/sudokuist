//! Sudoku puzzle generator with difficulty control
//!
//! This module generates valid Sudoku puzzles with specific difficulty targets
//! by using a combination of solution generation and selective clue removal.

use crate::types::{SolvingTechnique, DifficultyLevel, DifficultyAnalysis, BOARD_SIZE};
use crate::validator::has_unique_solution;
use crate::difficulty::analyze_difficulty;
use rand::{thread_rng, Rng};

/// Configuration for puzzle generation
#[derive(Debug, Clone)]
pub struct GeneratorConfig {
    /// Target difficulty level for generated puzzles
    pub target_difficulty: DifficultyLevel,
    /// Maximum number of attempts to generate a puzzle
    pub max_attempts: u32,
    /// Minimum number of clues to provide
    pub min_clues: usize,
    /// Maximum number of clues to provide
    pub max_clues: usize,
    /// Whether to optimize for aesthetic symmetry
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
    ///
    /// # Arguments
    /// * `config` - Configuration parameters for generation
    ///
    /// # Returns
    /// A new `PuzzleGenerator` instance
    pub fn new(config: GeneratorConfig) -> Self {
        Self { config }
    }

    /// Creates a generator with default settings for a specific difficulty
    ///
    /// # Arguments
    /// * `difficulty` - Target difficulty level
    ///
    /// # Returns
    /// A new `PuzzleGenerator` configured for the given difficulty
    pub fn with_difficulty(difficulty: DifficultyLevel) -> Self {
        let mut config = GeneratorConfig::default();
        config.target_difficulty = difficulty;
        
        // Adjust clue range based on difficulty
        match difficulty {
            DifficultyLevel::Easy => {
                config.min_clues = 35;
                config.max_clues = 45;
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
    ///
    /// This is the main entry point for puzzle generation. It attempts to
    /// create a puzzle that matches the target difficulty within the
    /// specified number of attempts.
    ///
    /// # Returns
    /// `Some(board)` if generation succeeded, `None` if it failed
    pub fn generate(&self) -> Option<Vec<Option<u8>>> {
        for attempt in 0..self.config.max_attempts {
            if let Some(puzzle) = self.generate_attempt() {
                // Verify the puzzle meets our requirements
                if self.validate_puzzle(&puzzle) {
                    return Some(puzzle);
                }
            }
            
            // Log progress occasionally
            if attempt % 100 == 0 && attempt > 0 {
                web_sys::console::log_1(&format!("Generation attempt {}", attempt).into());
            }
        }

        None
    }

    /// Attempts to generate a single puzzle
    ///
    /// # Returns
    /// `Some(puzzle)` if a valid puzzle was created, `None` otherwise
    fn generate_attempt(&self) -> Option<Vec<Option<u8>>> {
        // Step 1: Generate a complete, valid solution
        let solution = self.generate_complete_solution()?;

        // Step 2: Remove clues strategically to create the puzzle
        self.create_puzzle_from_solution(&solution)
    }

    /// Generates a complete, valid Sudoku solution
    ///
    /// Uses a randomized backtracking approach to create a fully filled
    /// grid that satisfies all Sudoku constraints.
    ///
    /// # Returns
    /// `Some(solution)` if generation succeeded, `None` otherwise
    fn generate_complete_solution(&self) -> Option<Vec<Option<u8>>> {
        let mut board = vec![None; BOARD_SIZE];
        
        if self.fill_board_randomly(&mut board, 0) {
            Some(board)
        } else {
            None
        }
    }

    /// Recursively fills the board using randomized backtracking
    ///
    /// # Arguments
    /// * `board` - The board to fill
    /// * `index` - Current cell index to fill
    ///
    /// # Returns
    /// `true` if the board was successfully filled
    fn fill_board_randomly(&self, board: &mut Vec<Option<u8>>, index: usize) -> bool {
        if index >= BOARD_SIZE {
            return true; // Board is complete
        }

        // Generate numbers 1-9 in random order
        let mut numbers: Vec<u8> = (1..=9).collect();
        let mut rng = thread_rng();
        
        // Shuffle the numbers for randomness
        for i in (1..numbers.len()).rev() {
            let j = rng.gen_range(0..=i);
            numbers.swap(i, j);
        }

        // Try each number in random order
        for &num in &numbers {
            if self.is_valid_placement(board, index, num) {
                board[index] = Some(num);
                
                if self.fill_board_randomly(board, index + 1) {
                    return true;
                }
                
                board[index] = None; // Backtrack
            }
        }

        false
    }

    /// Checks if placing a number at a position is valid
    ///
    /// # Arguments
    /// * `board` - The current board state
    /// * `index` - The cell index to check
    /// * `num` - The number to place
    ///
    /// # Returns
    /// `true` if the placement is valid according to Sudoku rules
    fn is_valid_placement(&self, board: &[Option<u8>], index: usize, num: u8) -> bool {
        let row = index / 9;
        let col = index % 9;

        // Check row
        for c in 0..9 {
            let check_index = row * 9 + c;
            if board[check_index] == Some(num) {
                return false;
            }
        }

        // Check column
        for r in 0..9 {
            let check_index = r * 9 + col;
            if board[check_index] == Some(num) {
                return false;
            }
        }

        // Check 3x3 box
        let box_row = (row / 3) * 3;
        let box_col = (col / 3) * 3;
        for r in box_row..box_row + 3 {
            for c in box_col..box_col + 3 {
                let check_index = r * 9 + c;
                if board[check_index] == Some(num) {
                    return false;
                }
            }
        }

        true
    }

    /// Creates a puzzle by strategically removing clues from a complete solution
    ///
    /// # Arguments
    /// * `solution` - A complete, valid Sudoku solution
    ///
    /// # Returns
    /// `Some(puzzle)` if clue removal succeeded, `None` otherwise
    fn create_puzzle_from_solution(&self, solution: &[Option<u8>]) -> Option<Vec<Option<u8>>> {
        let mut puzzle = solution.to_vec();
        let removal_order = self.get_removal_order();

        // Remove clues one by one, checking constraints
        for &index in &removal_order {
            let original_value = puzzle[index];
            puzzle[index] = None;

            // Check if puzzle still has unique solution and meets difficulty
            if !has_unique_solution(&puzzle) || !self.meets_clue_constraints(&puzzle) {
                puzzle[index] = original_value; // Restore the clue
            }

            // Early exit if we've reached minimum clues
            let clue_count = puzzle.iter().filter(|&&cell| cell.is_some()).count();
            if clue_count <= self.config.min_clues {
                break;
            }
        }

        Some(puzzle)
    }

    /// Generates the order in which to attempt clue removal
    ///
    /// Can prioritize symmetric patterns if configured to do so.
    ///
    /// # Returns
    /// A vector of cell indices in removal order
    fn get_removal_order(&self) -> Vec<usize> {
        let mut indices: Vec<usize> = (0..BOARD_SIZE).collect();
        let mut rng = thread_rng();

        if self.config.prefer_symmetry {
            // Group symmetric pairs and shuffle
            let mut pairs = Vec::new();
            let mut used = vec![false; BOARD_SIZE];

            for i in 0..BOARD_SIZE {
                if !used[i] {
                    let symmetric_index = self.get_symmetric_index(i);
                    if symmetric_index != i && !used[symmetric_index] {
                        pairs.push((i, symmetric_index));
                        used[i] = true;
                        used[symmetric_index] = true;
                    } else {
                        pairs.push((i, i)); // Center or already paired
                        used[i] = true;
                    }
                }
            }

            // Shuffle pairs and flatten
            for i in (1..pairs.len()).rev() {
                let j = rng.gen_range(0..=i);
                pairs.swap(i, j);
            }

            indices = pairs.into_iter().flat_map(|(a, b)| {
                if a == b { vec![a] } else { vec![a, b] }
            }).collect();
        } else {
            // Simple random shuffle
            for i in (1..indices.len()).rev() {
                let j = rng.gen_range(0..=i);
                indices.swap(i, j);
            }
        }

        indices
    }

    /// Gets the symmetric counterpart of a cell index
    ///
    /// Uses 180-degree rotational symmetry around the center.
    ///
    /// # Arguments
    /// * `index` - The original cell index
    ///
    /// # Returns
    /// The index of the symmetric counterpart
    fn get_symmetric_index(&self, index: usize) -> usize {
        let row = index / 9;
        let col = index % 9;
        let sym_row = 8 - row;
        let sym_col = 8 - col;
        sym_row * 9 + sym_col
    }

    /// Checks if the puzzle meets the clue count constraints
    ///
    /// # Arguments
    /// * `puzzle` - The puzzle to check
    ///
    /// # Returns
    /// `true` if the clue count is within acceptable range
    fn meets_clue_constraints(&self, puzzle: &[Option<u8>]) -> bool {
        let clue_count = puzzle.iter().filter(|&&cell| cell.is_some()).count();
        clue_count >= self.config.min_clues && clue_count <= self.config.max_clues
    }

    /// Validates that a generated puzzle meets all requirements
    ///
    /// # Arguments
    /// * `puzzle` - The puzzle to validate
    ///
    /// # Returns
    /// `true` if the puzzle meets all configured requirements
    fn validate_puzzle(&self, puzzle: &[Option<u8>]) -> bool {
        // Check basic constraints
        if !has_unique_solution(puzzle) || !self.meets_clue_constraints(puzzle) {
            return false;
        }

        // Check difficulty level
        let analysis = analyze_difficulty(puzzle);
        self.difficulty_matches_target(&analysis)
    }

    /// Checks if the analyzed difficulty matches the target
    ///
    /// # Arguments
    /// * `analysis` - The difficulty analysis result
    ///
    /// # Returns
    /// `true` if the difficulty is acceptable for the target
    fn difficulty_matches_target(&self, analysis: &DifficultyAnalysis) -> bool {
        // Allow some flexibility in difficulty matching
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
}

/// Utility function to generate a puzzle with specific difficulty
///
/// This is a convenience function for quickly generating puzzles without
/// needing to create a generator instance.
///
/// # Arguments
/// * `difficulty` - The target difficulty level
///
/// # Returns
/// `Some(puzzle)` if generation succeeded, `None` otherwise
///
/// # Example
/// ```rust,no_run
/// use sudoku_wasm::generator::generate_puzzle;
/// use sudoku_wasm::types::DifficultyLevel;
/// 
/// if let Some(puzzle) = generate_puzzle(DifficultyLevel::Medium) {
///     println!("Generated a medium difficulty puzzle!");
/// }
/// ```
pub fn generate_puzzle(difficulty: DifficultyLevel) -> Option<Vec<Option<u8>>> {
    let generator = PuzzleGenerator::with_difficulty(difficulty);
    generator.generate()
}

/// Utility function to generate multiple puzzles
///
/// Generates a batch of puzzles with the same difficulty level.
///
/// # Arguments
/// * `difficulty` - The target difficulty level
/// * `count` - Number of puzzles to generate
///
/// # Returns
/// A vector of successfully generated puzzles
pub fn generate_multiple_puzzles(difficulty: DifficultyLevel, count: usize) -> Vec<Vec<Option<u8>>> {
    let generator = PuzzleGenerator::with_difficulty(difficulty);
    let mut puzzles = Vec::new();

    for _ in 0..count {
        if let Some(puzzle) = generator.generate() {
            puzzles.push(puzzle);
        }
    }

    puzzles
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generator_creation() {
        let generator = PuzzleGenerator::with_difficulty(DifficultyLevel::Easy);
        assert_eq!(generator.config.target_difficulty, DifficultyLevel::Easy);
    }

    #[test]
    fn test_symmetric_index_calculation() {
        let generator = PuzzleGenerator::with_difficulty(DifficultyLevel::Medium);
        
        // Test corners
        assert_eq!(generator.get_symmetric_index(0), 80);  // Top-left -> Bottom-right
        assert_eq!(generator.get_symmetric_index(8), 72);  // Top-right -> Bottom-left
        assert_eq!(generator.get_symmetric_index(72), 8);  // Bottom-left -> Top-right
        assert_eq!(generator.get_symmetric_index(80), 0);  // Bottom-right -> Top-left
        
        // Test center
        assert_eq!(generator.get_symmetric_index(40), 40); // Center -> Center
    }

    #[test]
    fn test_clue_constraints() {
        let generator = PuzzleGenerator::with_difficulty(DifficultyLevel::Easy);
        
        // Create test puzzles with different clue counts
        let too_few_clues = vec![None; BOARD_SIZE];
        let good_clues: Vec<Option<u8>> = vec![Some(1); 40].into_iter()
            .chain(vec![None; BOARD_SIZE - 40])
            .collect();
        
        assert!(!generator.meets_clue_constraints(&too_few_clues));
        assert!(generator.meets_clue_constraints(&good_clues));
    }
}
