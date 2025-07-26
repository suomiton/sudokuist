//! Puzzle difficulty analysis and classification
//!
//! This module analyzes Sudoku puzzles to determine their difficulty level
//! based on the solving techniques required and other complexity metrics.

use crate::types::{DifficultyLevel, DifficultyAnalysis, SolvingTechnique};
use crate::solver::HumanStyleSolver;

/// Analyzes the difficulty of a Sudoku puzzle
///
/// Uses a human-style solver to determine what techniques are required
/// to solve the puzzle and classifies the overall difficulty.
///
/// # Arguments
/// * `board` - The puzzle board to analyze
///
/// # Returns
/// A `DifficultyAnalysis` containing difficulty metrics
pub fn analyze_difficulty(board: &[Option<u8>]) -> DifficultyAnalysis {
    let mut solver = HumanStyleSolver::new(board);
    let _solved = solver.solve_with_techniques();
    
    let hardest_technique = solver.get_hardest_technique_used();
    let techniques_used = solver.get_techniques_used();
    let branching_factor = solver.calculate_branching_factor();
    
    let level = classify_difficulty_level(&hardest_technique, techniques_used.len(), branching_factor);
    
    DifficultyAnalysis {
        level,
        hardest_technique,
        technique_diversity: techniques_used.len(),
        branching_factor,
    }
}

/// Classifies the difficulty level based on solving requirements
fn classify_difficulty_level(
    hardest_technique: &SolvingTechnique,
    technique_count: usize,
    _branching_factor: f64,
) -> DifficultyLevel {
    match hardest_technique {
        SolvingTechnique::NakedSingle | SolvingTechnique::HiddenSingle => DifficultyLevel::Easy,
        
        SolvingTechnique::NakedPair | SolvingTechnique::HiddenPair => {
            if technique_count <= 3 {
                DifficultyLevel::Medium
            } else {
                DifficultyLevel::Hard
            }
        }
        
        SolvingTechnique::BoxLineReduction | SolvingTechnique::PointingPairs => {
            DifficultyLevel::Medium
        }
        
        SolvingTechnique::XWing | SolvingTechnique::PointingTriples => {
            if technique_count <= 5 {
                DifficultyLevel::Hard
            } else {
                DifficultyLevel::Expert
            }
        }
        
        SolvingTechnique::Swordfish 
        | SolvingTechnique::Coloring
        | SolvingTechnique::XYWing
        | SolvingTechnique::XYChain
        | SolvingTechnique::ForcingChain
        | SolvingTechnique::TrialAndError => DifficultyLevel::Expert,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_difficulty_classification() {
        // Test basic technique classification
        let easy_level = classify_difficulty_level(&SolvingTechnique::NakedSingle, 1, 1.5);
        assert_eq!(easy_level, DifficultyLevel::Easy);
        
        let hard_level = classify_difficulty_level(&SolvingTechnique::XWing, 4, 3.0);
        assert_eq!(hard_level, DifficultyLevel::Hard);
        
        let expert_level = classify_difficulty_level(&SolvingTechnique::Swordfish, 6, 4.0);
        assert_eq!(expert_level, DifficultyLevel::Expert);
    }
}
