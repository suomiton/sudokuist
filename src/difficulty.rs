//! Puzzle difficulty analysis and classification
//!
//! This module analyzes Sudoku puzzles to determine their difficulty level
//! based on the solving techniques required and other complexity metrics.

use crate::grid::{coords_to_index, index_to_coords};
use crate::solver::HumanStyleSolver;
use crate::types::{
    CandidateGrid, DifficultyAnalysis, DifficultyLevel, SolvingTechnique, BOARD_SIZE, BOX_SIZE,
    GRID_SIZE,
};

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

    // If only basic techniques were found, use heuristic analysis but do not
    // promote to advanced techniques without actual solver usage.
    let hardest_technique = if basic_technique <= SolvingTechnique::HiddenSingle {
        let heuristic = analyze_difficulty_heuristic(board);
        if heuristic >= SolvingTechnique::XWing {
            basic_technique
        } else {
            heuristic
        }
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
    let single_fill_count = count_naked_single_propagation(board);
    let empty_cells = BOARD_SIZE - clue_count;
    let single_fill_ratio = if empty_cells > 0 {
        single_fill_count as f64 / empty_cells as f64
    } else {
        1.0
    };

    // Heuristic based on clue count, visibility of singles, and complexity
    if clue_count >= 50 || single_fill_ratio >= 0.70 {
        SolvingTechnique::NakedSingle
    } else if clue_count >= 42 || single_fill_ratio >= 0.55 {
        SolvingTechnique::HiddenSingle
    } else if clue_count >= 36 || single_fill_ratio >= 0.40 {
        SolvingTechnique::HiddenPair
    } else if clue_count >= 32 || single_fill_ratio >= 0.25 {
        if complexity > 3.1 {
            SolvingTechnique::NakedPair
        } else {
            SolvingTechnique::HiddenPair
        }
    } else if clue_count >= 30 {
        if complexity > 3.4 || single_fill_ratio < 0.15 {
            SolvingTechnique::BoxLineReduction
        } else {
            SolvingTechnique::NakedPair
        }
    } else {
        match clue_count {
            // Hard range: 25-29 clues - challenging
            28..=29 => SolvingTechnique::BoxLineReduction,
            27 => {
                if complexity > 2.9 || single_fill_ratio < 0.12 {
                    SolvingTechnique::PointingPairs
                } else {
                    SolvingTechnique::BoxLineReduction
                }
            }
            26 => SolvingTechnique::PointingPairs,
            25 => {
                if complexity > 3.3 || single_fill_ratio < 0.08 {
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

/// Counts how many naked singles can be placed after basic candidate propagation.
fn count_naked_single_propagation(board: &[Option<u8>]) -> usize {
    let mut working_board = board.to_vec();
    let mut candidates = CandidateGrid::new();

    for index in 0..BOARD_SIZE {
        if let Some(num) = working_board[index] {
            candidates.set_only_candidate(index, num);
            eliminate_candidates_in_units(&mut candidates, index, num);
        }
    }

    let mut placements = 0;
    loop {
        let mut progress = false;

        for index in 0..BOARD_SIZE {
            if working_board[index].is_none() && candidates.candidate_count(index) == 1 {
                if let Some(&num) = candidates.get_candidates(index).first() {
                    working_board[index] = Some(num);
                    candidates.set_only_candidate(index, num);
                    eliminate_candidates_in_units(&mut candidates, index, num);
                    placements += 1;
                    progress = true;
                }
            }
        }

        if !progress {
            break;
        }
    }

    placements
}

fn eliminate_candidates_in_units(candidates: &mut CandidateGrid, index: usize, num: u8) {
    let (row, col) = index_to_coords(index);

    for i in 0..GRID_SIZE {
        let row_idx = coords_to_index(row, i);
        let col_idx = coords_to_index(i, col);
        candidates.remove_candidate(row_idx, num);
        candidates.remove_candidate(col_idx, num);
    }

    let box_start_row = (row / BOX_SIZE) * BOX_SIZE;
    let box_start_col = (col / BOX_SIZE) * BOX_SIZE;
    for r in box_start_row..box_start_row + BOX_SIZE {
        for c in box_start_col..box_start_col + BOX_SIZE {
            let box_idx = coords_to_index(r, c);
            candidates.remove_candidate(box_idx, num);
        }
    }
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
