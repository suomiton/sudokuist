//! Puzzle difficulty analysis and classification
//!
//! This module analyzes Sudoku puzzles to determine their difficulty level
//! based on the solving techniques required and other complexity metrics.

use crate::solver::HumanStyleSolver;
use crate::types::{DifficultyAnalysis, DifficultyLevel, SolvingTechnique};

/// Analyzes the difficulty of a Sudoku puzzle
///
/// Uses both human-style solver (for basic techniques) and heuristic analysis
/// to determine difficulty when advanced techniques are not implemented.
///
/// # Arguments
/// * `board` - The puzzle board to analyze
///
/// # Returns
/// A `DifficultyAnalysis` containing difficulty metrics
pub fn analyze_difficulty(board: &[Option<u8>]) -> DifficultyAnalysis {
    let mut solver = HumanStyleSolver::new(board);
    let _solved = solver.solve_with_techniques();

    let basic_technique = solver.get_hardest_technique_used();
    let techniques_used = solver.get_techniques_used();
    let branching_factor = solver.calculate_branching_factor();

    // If only basic techniques were found, use heuristic analysis for advanced puzzles
    let hardest_technique = if basic_technique <= SolvingTechnique::HiddenSingle {
        analyze_difficulty_heuristic(board)
    } else {
        basic_technique
    };

    let level =
        classify_difficulty_level(&hardest_technique, techniques_used.len(), branching_factor);

    DifficultyAnalysis {
        level,
        hardest_technique,
        technique_diversity: techniques_used.len(),
        branching_factor,
    }
}

/// Heuristic-based difficulty analysis for when advanced solver techniques are not implemented
///
/// Uses puzzle characteristics like clue count, constraint density, and solving complexity
/// to estimate the difficulty level and required techniques.
fn analyze_difficulty_heuristic(board: &[Option<u8>]) -> SolvingTechnique {
    let clue_count = board.iter().filter(|c| c.is_some()).count();
    let complexity = calculate_puzzle_complexity(board);

    // Heuristic based on clue count and complexity
    match clue_count {
        45..=81 => SolvingTechnique::NakedSingle,
        36..=44 => SolvingTechnique::HiddenSingle,

        // Medium range: 30-35 clues - moderate difficulty
        34..=35 => SolvingTechnique::HiddenPair,
        32..=33 => {
            if complexity > 2.8 {
                SolvingTechnique::NakedPair
            } else {
                SolvingTechnique::HiddenPair
            }
        }
        30..=31 => SolvingTechnique::NakedPair,

        // Hard range: 25-29 clues - challenging
        28..=29 => SolvingTechnique::BoxLineReduction,
        27 => {
            if complexity > 2.8 {
                SolvingTechnique::PointingPairs
            } else {
                SolvingTechnique::BoxLineReduction
            }
        }
        26 => SolvingTechnique::PointingPairs,
        25 => {
            if complexity > 3.2 {
                SolvingTechnique::XWing
            } else {
                SolvingTechnique::PointingPairs
            }
        }

        // Very Hard range: 17-24 clues - expert level
        22..=24 => {
            if complexity > 3.8 {
                SolvingTechnique::Swordfish
            } else {
                SolvingTechnique::XWing
            }
        }
        20..=21 => SolvingTechnique::Swordfish,
        18..=19 => {
            if complexity > 4.2 {
                SolvingTechnique::XYWing
            } else {
                SolvingTechnique::Swordfish
            }
        }
        17 => {
            if complexity > 4.5 {
                SolvingTechnique::XYChain
            } else {
                SolvingTechnique::XYWing
            }
        }

        _ => SolvingTechnique::ForcingChain,
    }
}

/// Calculates a complexity metric for the puzzle based on constraint density
fn calculate_puzzle_complexity(board: &[Option<u8>]) -> f64 {
    let mut complexity = 0.0;
    let mut constraint_density = 0.0;

    // Calculate how "spread out" the clues are
    for i in 0..81 {
        if board[i].is_some() {
            let row = i / 9;
            let col = i % 9;
            let box_start = (row / 3) * 3 * 9 + (col / 3) * 3;

            // Count neighbors in same row
            let row_clues = (0..9).filter(|&c| board[row * 9 + c].is_some()).count();

            // Count neighbors in same column
            let col_clues = (0..9).filter(|&r| board[r * 9 + col].is_some()).count();

            // Count neighbors in same box
            let box_clues = (0..3)
                .flat_map(|r| (0..3).map(move |c| box_start + r * 9 + c))
                .filter(|&idx| board[idx].is_some())
                .count();

            // Higher density in same units = lower complexity
            constraint_density += (row_clues + col_clues + box_clues) as f64;
        }
    }

    let clue_count = board.iter().filter(|c| c.is_some()).count();
    if clue_count > 0 {
        complexity = 3.0 - (constraint_density / (clue_count as f64 * 3.0))
            + (35.0 - clue_count as f64) / 10.0;
    }

    complexity.max(1.0).min(5.0)
}

/// Classifies the difficulty level based on solving requirements
fn classify_difficulty_level(
    hardest_technique: &SolvingTechnique,
    technique_count: usize,
    branching_factor: f64,
) -> DifficultyLevel {
    // Use branching factor as a secondary classifier
    let bf_difficulty = match branching_factor {
        bf if bf <= 1.7 => DifficultyLevel::VeryEasy,
        bf if bf <= 2.2 => DifficultyLevel::Easy,
        bf if bf <= 3.8 => DifficultyLevel::Medium,
        bf if bf <= 6.0 => DifficultyLevel::Hard,
        _ => DifficultyLevel::Expert,
    };

    // Primary classification by technique
    let technique_difficulty = match hardest_technique {
        SolvingTechnique::NakedSingle => {
            if branching_factor <= 1.7 {
                DifficultyLevel::VeryEasy
            } else {
                DifficultyLevel::Easy
            }
        }
        SolvingTechnique::HiddenSingle => DifficultyLevel::Easy,

        SolvingTechnique::NakedPair | SolvingTechnique::HiddenPair => {
            if technique_count <= 5 && branching_factor <= 3.5 {
                DifficultyLevel::Medium
            } else {
                DifficultyLevel::Hard
            }
        }

        SolvingTechnique::BoxLineReduction | SolvingTechnique::PointingPairs => {
            if branching_factor <= 4.2 {
                DifficultyLevel::Medium
            } else {
                DifficultyLevel::Hard
            }
        }

        SolvingTechnique::XWing | SolvingTechnique::PointingTriples => {
            if technique_count <= 7 && branching_factor <= 5.5 {
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
        | SolvingTechnique::TrialAndError => {
            if branching_factor <= 7.0 {
                DifficultyLevel::Hard
            } else {
                DifficultyLevel::Expert
            }
        }
    };

    // Return the higher difficulty between technique and branching factor
    // This ensures puzzles with high branching factor get proper classification
    match (technique_difficulty, bf_difficulty) {
        (DifficultyLevel::VeryEasy, _) => DifficultyLevel::VeryEasy,
        (DifficultyLevel::Easy, DifficultyLevel::VeryEasy) => DifficultyLevel::VeryEasy,
        (DifficultyLevel::Easy, bf) => bf,
        (DifficultyLevel::Medium, DifficultyLevel::VeryEasy | DifficultyLevel::Easy) => {
            DifficultyLevel::Medium
        }
        (DifficultyLevel::Medium, bf) => bf,
        (
            DifficultyLevel::Hard,
            DifficultyLevel::VeryEasy | DifficultyLevel::Easy | DifficultyLevel::Medium,
        ) => DifficultyLevel::Hard,
        (DifficultyLevel::Hard, bf) => bf,
        (DifficultyLevel::Expert, _) => DifficultyLevel::Expert,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_difficulty_classification() {
        // Test basic technique classification
        let very_easy_level = classify_difficulty_level(&SolvingTechnique::NakedSingle, 1, 1.5);
        assert_eq!(very_easy_level, DifficultyLevel::VeryEasy);

        let easy_level = classify_difficulty_level(&SolvingTechnique::NakedSingle, 1, 2.1);
        assert_eq!(easy_level, DifficultyLevel::Easy);

        let hard_level = classify_difficulty_level(&SolvingTechnique::XWing, 4, 3.0);
        assert_eq!(hard_level, DifficultyLevel::Hard);

        // Expert level with higher branching factor
        let expert_level = classify_difficulty_level(&SolvingTechnique::Swordfish, 6, 7.5);
        assert_eq!(expert_level, DifficultyLevel::Expert);
    }

    #[test]
    fn test_heuristic_analysis() {
        // Create a test puzzle with specific clue count
        let mut board = vec![None; 81];

        // Fill with 26 clues (should now be PointingPairs which is Medium-Hard)
        for i in 0..26 {
            board[i] = Some(1);
        }

        let technique = analyze_difficulty_heuristic(&board);
        println!("26 clues -> {:?}", technique);

        // Should be in Medium-Hard range (PointingPairs to XWing)
        assert!(
            technique >= SolvingTechnique::PointingPairs && technique <= SolvingTechnique::XWing
        );

        // Test with fewer clues for Hard level
        let mut hard_board = vec![None; 81];
        for i in 0..23 {
            hard_board[i] = Some(1);
        }

        let hard_technique = analyze_difficulty_heuristic(&hard_board);
        println!("23 clues -> {:?}", hard_technique);

        // Should be in Hard range (XWing to Swordfish)
        assert!(
            hard_technique >= SolvingTechnique::XWing
                && hard_technique <= SolvingTechnique::Swordfish
        );
    }
}
