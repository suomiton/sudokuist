//! Difficulty analysis and requirements for Sudoku puzzles
//!
//! This module defines difficulty levels and analyzes puzzle complexity
//! based on the solving techniques required to complete them.

use crate::types::{SolvingTechnique, DifficultyAnalysis};
use crate::solver::HumanStyleSolver;
use crate::validator::has_unique_solution;

/// Defines the requirements for each difficulty level
///
/// Returns a tuple containing:
/// - minimum clues required
/// - maximum clues allowed  
/// - list of techniques that should be sufficient
/// - maximum acceptable branching factor
///
/// # Arguments
/// * `level` - Difficulty level from 1 (very easy) to 5 (very hard)
///
/// # Returns
/// Tuple of (min_clues, max_clues, required_techniques, max_branching_factor)
pub fn get_difficulty_requirements(level: u8) -> (usize, usize, Vec<SolvingTechnique>, f64) {
    match level {
        1 => (
            38, // At least 38 given numbers (very approachable)
            45, // Up to 45 clues (lots of help)
            vec![
                SolvingTechnique::NakedSingle,
                SolvingTechnique::HiddenSingle,
            ],
            1.2, // Low branching factor (few choices per cell)
        ),
        2 => (
            32, // Roughly 32-37 clues (moderate help)
            37,
            vec![
                SolvingTechnique::NakedSingle,
                SolvingTechnique::HiddenSingle,
                SolvingTechnique::NakedPair,
                SolvingTechnique::BoxLineReduction,
            ],
            1.8, // Moderate branching factor
        ),
        3 => (
            28, // Roughly 28-31 clues (requires logical thinking)
            31,
            vec![
                SolvingTechnique::NakedSingle,
                SolvingTechnique::HiddenSingle,
                SolvingTechnique::NakedPair,
                SolvingTechnique::BoxLineReduction,
                SolvingTechnique::XWing,
                SolvingTechnique::PointingPairs,
            ],
            2.0, // Higher branching factor
        ),
        4 => (
            24, // Roughly 24-27 clues (advanced techniques needed)
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
            2.5, // High branching factor
        ),
        5 => (
            22, // Roughly 22-24 clues (expert level)
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
            2.0, // Target >= 2.0 branching factor (many choices)
        ),
        _ => (
            30, // Default fallback
            40,
            vec![SolvingTechnique::NakedSingle],
            1.5,
        ),
    }
}

/// Analyzes a puzzle to determine its actual difficulty level
///
/// Uses a human-style solver to determine what techniques are required
/// to solve the puzzle, then classifies it based on the hardest technique needed.
///
/// # Arguments
/// * `board` - The puzzle board with Some(num) for clues and None for empty cells
///
/// # Returns
/// A `DifficultyAnalysis` struct containing comprehensive difficulty information
pub fn analyze_difficulty(board: &[Option<u8>]) -> DifficultyAnalysis {
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
    let difficulty_level = classify_difficulty_by_technique(&hardest_technique, clue_count, branching_factor);

    // Get techniques required based on what was actually used
    let techniques_required: Vec<String> = solver
        .get_techniques_used()
        .iter()
        .map(|t| format!("{:?}", t))
        .collect();

    let hardest_technique_name = format!("{:?}", hardest_technique);

    DifficultyAnalysis {
        level: difficulty_level,
        hardest_technique,
        technique_diversity: techniques_required.len(),
        branching_factor,
    }
}

/// Classifies difficulty level based on the hardest technique required
///
/// # Arguments
/// * `hardest_technique` - The most advanced technique needed to solve the puzzle
/// * `clue_count` - Number of given clues in the puzzle
/// * `branching_factor` - Average number of candidates per empty cell
///
/// # Returns
/// Difficulty level from 1 (very easy) to 5 (very hard)
fn classify_difficulty_by_technique(
    hardest_technique: &SolvingTechnique,
    clue_count: usize,
    branching_factor: f64,
) -> u8 {
    match hardest_technique {
        SolvingTechnique::NakedSingle | SolvingTechnique::HiddenSingle => {
            // Very easy if lots of clues and low branching factor
            if clue_count >= 38 && branching_factor <= 1.2 {
                1
            } else {
                2
            }
        }
        SolvingTechnique::NakedPair
        | SolvingTechnique::HiddenPair
        | SolvingTechnique::BoxLineReduction
        | SolvingTechnique::PointingPairs => 2, // Easy
        SolvingTechnique::XWing | SolvingTechnique::PointingTriples => 3, // Medium
        SolvingTechnique::Swordfish | SolvingTechnique::Coloring | SolvingTechnique::XYWing => 4, // Hard
        _ => 5, // Very Hard (advanced chains, trial and error)
    }
}

/// Validates that a puzzle meets the requirements for its target difficulty
///
/// # Arguments
/// * `board` - The puzzle board to validate
/// * `target_difficulty` - The intended difficulty level (1-5)
///
/// # Returns
/// `true` if the puzzle meets the difficulty requirements
pub fn validates_difficulty_requirements(board: &[Option<u8>], target_difficulty: u8) -> bool {
    let analysis = analyze_difficulty(board);
    let (min_clues, max_clues, required_techniques, max_branching) = 
        get_difficulty_requirements(target_difficulty);

    // Check clue count is within range
    if analysis.clue_count < min_clues || analysis.clue_count > max_clues {
        return false;
    }

    // Check branching factor is appropriate
    if target_difficulty <= 4 && analysis.branching_factor > max_branching {
        return false;
    }

    // For very hard puzzles, we want high branching factor
    if target_difficulty == 5 && analysis.branching_factor < max_branching {
        return false;
    }

    // Check that appropriate techniques are required
    let needs_advanced_technique = match target_difficulty {
        3 => analysis.hardest_technique.contains("Pair") || analysis.hardest_technique.contains("Wing"),
        4 => analysis.hardest_technique.contains("Wing") || analysis.hardest_technique.contains("Swordfish"),
        5 => analysis.hardest_technique.contains("Swordfish") 
            || analysis.hardest_technique.contains("Chain") 
            || analysis.hardest_technique.contains("XY"),
        _ => true, // Easy difficulties are more flexible
    };

    needs_advanced_technique && analysis.uniqueness_verified
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_difficulty_requirements() {
        let (min_clues, max_clues, techniques, branching) = get_difficulty_requirements(1);
        assert_eq!(min_clues, 38);
        assert_eq!(max_clues, 45);
        assert_eq!(branching, 1.2);
        assert!(techniques.contains(&SolvingTechnique::NakedSingle));
    }

    #[test]
    fn test_classify_difficulty_by_technique() {
        assert_eq!(
            classify_difficulty_by_technique(&SolvingTechnique::NakedSingle, 40, 1.1),
            1
        );
        assert_eq!(
            classify_difficulty_by_technique(&SolvingTechnique::XWing, 30, 2.0),
            3
        );
        assert_eq!(
            classify_difficulty_by_technique(&SolvingTechnique::XYWing, 25, 2.5),
            4
        );
    }
}
