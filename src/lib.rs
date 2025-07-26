//! Sudoku WASM library
//!
//! A comprehensive Sudoku puzzle generator and solver built in Rust and
//! compiled to WebAssembly for high-performance web applications.
//!
//! This library provides:
//! - Human-style puzzle solving with various logical techniques
//! - Difficulty-based puzzle generation
//! - Comprehensive board validation
//! - WebAssembly exports for JavaScript integration
//!
//! # Module Organization
//!
//! - [`types`] - Core type definitions and constants
//! - [`grid`] - Grid coordinate utilities and basic operations  
//! - [`validator`] - Board validation and constraint checking
//! - [`difficulty`] - Puzzle difficulty analysis and classification
//! - [`solver`] - Human-style solving with logical techniques
//! - [`generator`] - Puzzle generation with difficulty targeting
//! - [`wasm_exports`] - WebAssembly interface for JavaScript

// Module declarations
pub mod difficulty;
pub mod generator;
pub mod grid;
pub mod solver;
pub mod types;
pub mod validator;
pub mod wasm_exports;

// Re-export main functionality for easier access
pub use difficulty::analyze_difficulty;
pub use generator::{generate_puzzle, GeneratorConfig, PuzzleGenerator};
pub use solver::HumanStyleSolver;
pub use types::{DifficultyAnalysis, DifficultyLevel, SolvingTechnique, BOARD_SIZE, GRID_SIZE};
pub use validator::{has_unique_solution, solve_board, validate_board};

// Re-export WASM functions for direct access
pub use wasm_exports::*;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_module_integration() {
        // Test that all modules can work together
        let empty_board = vec![None; BOARD_SIZE];
        let result = validate_board(&empty_board);
        assert!(result.invalid_indices.is_empty());
        assert!(!result.is_complete);
    }

    #[test]
    fn test_basic_functionality() {
        // Test basic solver functionality
        let mut board = vec![None; BOARD_SIZE];
        // Place a simple number
        board[0] = Some(1);

        let solver = HumanStyleSolver::new(&board);
        assert!(!solver.is_solved());
    }

    #[test]
    fn test_constants() {
        assert_eq!(BOARD_SIZE, 81);
        assert_eq!(GRID_SIZE, 9);
    }
}
