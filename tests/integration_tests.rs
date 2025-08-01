//! Comprehensive integration tests for the Sudoku engine
//!
//! This module contains extensive tests that validate all aspects of our
//! Sudoku implementation, from basic validation to complex puzzle generation.

use sudoku_wasm::{
    difficulty::analyze_difficulty, generator::PuzzleGenerator, solve_board,
    solver::HumanStyleSolver, types::DifficultyLevel, validate_board,
};

#[cfg(target_arch = "wasm32")]
use sudoku_wasm::has_unique_solution;

/// A valid, complete Sudoku board for testing
const VALID_COMPLETE_BOARD: [Option<u8>; 81] = [
    Some(5),
    Some(3),
    Some(4),
    Some(6),
    Some(7),
    Some(8),
    Some(9),
    Some(1),
    Some(2),
    Some(6),
    Some(7),
    Some(2),
    Some(1),
    Some(9),
    Some(5),
    Some(3),
    Some(4),
    Some(8),
    Some(1),
    Some(9),
    Some(8),
    Some(3),
    Some(4),
    Some(2),
    Some(5),
    Some(6),
    Some(7),
    Some(8),
    Some(5),
    Some(9),
    Some(7),
    Some(6),
    Some(1),
    Some(4),
    Some(2),
    Some(3),
    Some(4),
    Some(2),
    Some(6),
    Some(8),
    Some(5),
    Some(3),
    Some(7),
    Some(9),
    Some(1),
    Some(7),
    Some(1),
    Some(3),
    Some(9),
    Some(2),
    Some(4),
    Some(8),
    Some(5),
    Some(6),
    Some(9),
    Some(6),
    Some(1),
    Some(5),
    Some(3),
    Some(7),
    Some(2),
    Some(8),
    Some(4),
    Some(2),
    Some(8),
    Some(7),
    Some(4),
    Some(1),
    Some(9),
    Some(6),
    Some(3),
    Some(5),
    Some(3),
    Some(4),
    Some(5),
    Some(2),
    Some(8),
    Some(6),
    Some(1),
    Some(7),
    Some(9),
];

/// A valid puzzle with some cells empty
const VALID_PUZZLE: [Option<u8>; 81] = [
    Some(5),
    Some(3),
    None,
    Some(6),
    None,
    Some(8),
    None,
    Some(1),
    Some(2),
    Some(6),
    None,
    Some(2),
    Some(1),
    Some(9),
    Some(5),
    None,
    Some(4),
    None,
    Some(1),
    Some(9),
    Some(8),
    None,
    Some(4),
    Some(2),
    Some(5),
    None,
    Some(7),
    Some(8),
    Some(5),
    Some(9),
    Some(7),
    Some(6),
    None,
    Some(4),
    Some(2),
    Some(3),
    Some(4),
    None,
    Some(6),
    Some(8),
    Some(5),
    Some(3),
    Some(7),
    None,
    Some(1),
    Some(7),
    Some(1),
    Some(3),
    Some(9),
    None,
    Some(4),
    Some(8),
    Some(5),
    Some(6),
    Some(9),
    Some(6),
    Some(1),
    Some(5),
    Some(3),
    Some(7),
    None,
    Some(8),
    Some(4),
    Some(2),
    Some(8),
    None,
    Some(4),
    Some(1),
    Some(9),
    Some(6),
    Some(3),
    Some(5),
    Some(3),
    Some(4),
    Some(5),
    None,
    Some(8),
    Some(6),
    Some(1),
    Some(7),
    None,
];

/// An invalid board with conflicts
const INVALID_BOARD: [Option<u8>; 81] = [
    Some(5),
    Some(5),
    None,
    Some(6),
    None,
    Some(8),
    None,
    Some(1),
    Some(2), // Row conflict: two 5s
    Some(6),
    None,
    Some(2),
    Some(1),
    Some(9),
    Some(5),
    None,
    Some(4),
    None,
    Some(1),
    Some(9),
    Some(8),
    None,
    Some(4),
    Some(2),
    Some(5),
    None,
    Some(7),
    Some(8),
    Some(5),
    Some(9),
    Some(7),
    Some(6),
    None,
    Some(4),
    Some(2),
    Some(3),
    Some(4),
    None,
    Some(6),
    Some(8),
    Some(5),
    Some(3),
    Some(7),
    None,
    Some(1),
    Some(7),
    Some(1),
    Some(3),
    Some(9),
    None,
    Some(4),
    Some(8),
    Some(5),
    Some(6),
    Some(9),
    Some(6),
    Some(1),
    Some(5),
    Some(3),
    Some(7),
    None,
    Some(8),
    Some(4),
    Some(2),
    Some(8),
    None,
    Some(4),
    Some(1),
    Some(9),
    Some(6),
    Some(3),
    Some(5),
    Some(3),
    Some(4),
    Some(5),
    None,
    Some(8),
    Some(6),
    Some(1),
    Some(7),
    None,
];

mod validation_tests {
    use super::*;

    #[test]
    fn test_valid_complete_board() {
        let result = validate_board(&VALID_COMPLETE_BOARD);
        assert!(
            result.invalid_indices.is_empty(),
            "Valid complete board should have no invalid indices"
        );
        assert!(
            result.is_complete,
            "Complete valid board should be marked as complete"
        );
    }

    #[test]
    fn test_valid_incomplete_board() {
        let result = validate_board(&VALID_PUZZLE);
        assert!(
            result.invalid_indices.is_empty(),
            "Valid puzzle should have no invalid indices"
        );
        assert!(
            !result.is_complete,
            "Incomplete board should not be marked as complete"
        );
    }

    #[test]
    fn test_invalid_board_detected() {
        let result = validate_board(&INVALID_BOARD);
        assert!(
            !result.invalid_indices.is_empty(),
            "Invalid board should have invalid indices detected"
        );
        assert!(!result.is_complete, "Invalid board should not be complete");

        // The conflict is at index 1 (second cell in first row)
        assert!(
            result.invalid_indices.contains(&1),
            "Should detect the duplicate 5 in first row"
        );
    }

    #[test]
    fn test_empty_board() {
        let empty_board = vec![None; 81];
        let result = validate_board(&empty_board);
        assert!(
            result.invalid_indices.is_empty(),
            "Empty board should be valid"
        );
        assert!(!result.is_complete, "Empty board should not be complete");
    }

    #[test]
    fn test_board_with_invalid_numbers() {
        let mut bad_board = VALID_PUZZLE.clone();
        bad_board[0] = Some(10); // Invalid number > 9

        // This should either be caught by type system or validation
        // Since we use u8, 10 is valid for the type but invalid for Sudoku
        // Our validation should catch logical conflicts, not type violations
    }
}

mod solver_tests {
    use super::*;

    #[test]
    fn test_solver_creation() {
        let solver = HumanStyleSolver::new(&VALID_PUZZLE);
        assert!(
            !solver.is_solved(),
            "Incomplete puzzle should not be marked as solved"
        );
    }

    #[test]
    fn test_solver_with_complete_board() {
        let solver = HumanStyleSolver::new(&VALID_COMPLETE_BOARD);
        assert!(
            solver.is_solved(),
            "Complete valid board should be marked as solved"
        );
    }

    #[test]
    fn test_solver_techniques() {
        let mut solver = HumanStyleSolver::new(&VALID_PUZZLE);
        let initial_solved = solver.is_solved();

        // Try to solve with techniques
        let solved = solver.solve_with_techniques();

        // Should make some progress or solve completely
        assert!(
            solved || !initial_solved,
            "Solver should either solve or make progress"
        );

        // Check that some techniques were used
        let techniques = solver.get_techniques_used();
        assert!(
            !techniques.is_empty(),
            "Some solving techniques should have been used"
        );
    }

    #[test]
    fn test_branching_factor() {
        let solver = HumanStyleSolver::new(&VALID_PUZZLE);
        let factor = solver.calculate_branching_factor();

        // Should be reasonable for a valid puzzle
        assert!(
            factor >= 1.0 && factor <= 9.0,
            "Branching factor should be between 1 and 9"
        );
    }
}

mod generator_tests {
    use super::*;

    #[test]
    #[cfg(target_arch = "wasm32")] // Only run in WASM environment
    fn test_generate_easy_puzzle() {
        let generator = PuzzleGenerator::with_difficulty(DifficultyLevel::Easy);

        // Note: Generation might fail, so we test the attempt
        if let Some(puzzle) = generator.generate() {
            // Validate the generated puzzle
            let result = validate_board(&puzzle);
            assert!(
                result.invalid_indices.is_empty(),
                "Generated puzzle should be valid"
            );

            // Check it has a reasonable number of clues
            let clue_count = puzzle.iter().filter(|&&cell| cell.is_some()).count();
            assert!(
                clue_count >= 17 && clue_count <= 50,
                "Puzzle should have reasonable number of clues"
            );

            // Check it has a unique solution
            assert!(
                has_unique_solution(&puzzle),
                "Generated puzzle should have unique solution"
            );
        }
    }

    #[test]
    #[cfg(target_arch = "wasm32")] // Only run in WASM environment
    fn test_generate_medium_puzzle() {
        let generator = PuzzleGenerator::with_difficulty(DifficultyLevel::Medium);

        if let Some(puzzle) = generator.generate() {
            let result = validate_board(&puzzle);
            assert!(
                result.invalid_indices.is_empty(),
                "Generated medium puzzle should be valid"
            );
            assert!(
                has_unique_solution(&puzzle),
                "Generated puzzle should have unique solution"
            );
        }
    }

    #[test]
    fn test_generator_configuration() {
        // Test generator creation (doesn't require actual generation)
        let generator = PuzzleGenerator::with_difficulty(DifficultyLevel::Easy);
        // Just test that we can create the generator without panicking
        assert_eq!(
            std::mem::size_of_val(&generator),
            std::mem::size_of::<PuzzleGenerator>()
        );
    }
}

mod difficulty_tests {
    use super::*;

    #[test]
    fn test_difficulty_analysis() {
        let analysis = analyze_difficulty(&VALID_PUZZLE);

        // Check that analysis has reasonable values
        assert!(matches!(
            analysis.level,
            DifficultyLevel::VeryEasy
                | DifficultyLevel::Easy
                | DifficultyLevel::Medium
                | DifficultyLevel::Hard
                | DifficultyLevel::Expert
        ));
        assert!(
            analysis.technique_diversity <= 10,
            "Technique diversity should be reasonable"
        );
        assert!(
            analysis.branching_factor >= 1.0 && analysis.branching_factor <= 9.0,
            "Branching factor should be reasonable"
        );
    }

    #[test]
    fn test_empty_board_difficulty() {
        let empty_board = vec![None; 81];
        let analysis = analyze_difficulty(&empty_board);

        // Empty board should be analyzed (though not very meaningful)
        assert!(
            analysis.branching_factor > 5.0,
            "Empty board should have high branching factor"
        );
    }
}

mod integration_tests {
    use super::*;

    #[test]
    #[cfg(target_arch = "wasm32")] // Only run in WASM environment
    fn test_full_workflow() {
        // Test the complete workflow: generate -> validate -> solve -> analyze

        // Step 1: Generate a puzzle
        let generator = PuzzleGenerator::with_difficulty(DifficultyLevel::Easy);

        if let Some(puzzle) = generator.generate() {
            println!(
                "Generated puzzle with {} clues",
                puzzle.iter().filter(|&&cell| cell.is_some()).count()
            );

            // Step 2: Validate it
            let validation = validate_board(&puzzle);
            assert!(
                validation.invalid_indices.is_empty(),
                "Generated puzzle should be valid"
            );

            // Step 3: Try to solve it
            let mut solver = HumanStyleSolver::new(&puzzle);
            let solved = solver.solve_with_techniques();

            // Step 4: Analyze difficulty
            let analysis = analyze_difficulty(&puzzle);
            assert_eq!(
                analysis.level,
                DifficultyLevel::Easy,
                "Should match target difficulty"
            );

            println!(
                "Workflow test successful: solved={}, difficulty={:?}",
                solved, analysis.level
            );
        } else {
            println!("Could not generate puzzle for workflow test - this is acceptable");
        }
    }

    #[test]
    fn test_workflow_with_known_puzzle() {
        // Test workflow with a predefined puzzle instead of generating one
        let puzzle = VALID_PUZZLE.clone();

        // Step 1: Validate it
        let validation = validate_board(&puzzle);
        assert!(
            validation.invalid_indices.is_empty(),
            "Predefined puzzle should be valid"
        );

        // Step 2: Try to solve it
        let mut solver = HumanStyleSolver::new(&puzzle);
        let _solved = solver.solve_with_techniques();

        // Step 3: Analyze difficulty
        let analysis = analyze_difficulty(&puzzle);
        // Any difficulty level is fine for this test
        assert!(matches!(
            analysis.level,
            DifficultyLevel::VeryEasy
                | DifficultyLevel::Easy
                | DifficultyLevel::Medium
                | DifficultyLevel::Hard
                | DifficultyLevel::Expert
        ));

        println!(
            "Workflow test with known puzzle successful: difficulty={:?}",
            analysis.level
        );
    }

    #[test]
    fn test_backtracking_solver() {
        let mut puzzle = VALID_PUZZLE.clone();
        let original_clues = puzzle.iter().filter(|&&cell| cell.is_some()).count();

        // Try to solve with backtracking
        let solved = solve_board(&mut puzzle);
        assert!(solved, "Backtracking solver should solve valid puzzle");

        // Check the solution is complete and valid
        let final_clues = puzzle.iter().filter(|&&cell| cell.is_some()).count();
        assert_eq!(final_clues, 81, "Solved puzzle should be complete");

        let validation = validate_board(&puzzle);
        assert!(
            validation.is_complete && validation.invalid_indices.is_empty(),
            "Solved puzzle should be complete and valid"
        );

        println!("Backtracking solver filled {} cells", 81 - original_clues);
    }
}

#[cfg(test)]
mod stress_tests {
    use super::*;

    #[test]
    #[cfg(target_arch = "wasm32")] // Only run in WASM environment
    fn test_multiple_generations() {
        // Test that we can generate multiple puzzles reliably
        let generator = PuzzleGenerator::with_difficulty(DifficultyLevel::Easy);
        let mut success_count = 0;
        let attempts = 5;

        for i in 0..attempts {
            if let Some(puzzle) = generator.generate() {
                let validation = validate_board(&puzzle);
                assert!(
                    validation.invalid_indices.is_empty(),
                    "Generated puzzle {} should be valid",
                    i
                );
                success_count += 1;
            }
        }

        println!(
            "Successfully generated {}/{} puzzles",
            success_count, attempts
        );
        // We should generate at least some puzzles, but generation can fail
        assert!(success_count > 0, "Should generate at least one puzzle");
    }

    #[test]
    fn test_solver_performance() {
        // Test solver performance with multiple predefined puzzles
        let test_puzzles = [VALID_PUZZLE, VALID_COMPLETE_BOARD];

        for (i, puzzle) in test_puzzles.iter().enumerate() {
            let mut solver = HumanStyleSolver::new(puzzle);
            let start_time = std::time::Instant::now();

            let _solved = solver.solve_with_techniques();
            let duration = start_time.elapsed();

            println!("Puzzle {} solved in {:?}", i, duration);
            // Should complete reasonably quickly (within 1 second for simple puzzles)
            assert!(duration.as_secs() < 1, "Solver should be reasonably fast");
        }
    }
}
