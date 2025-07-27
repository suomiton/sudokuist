//! Sudoku puzzle generator with difficulty control

use crate::difficulty::analyze_difficulty;
use crate::types::{DifficultyAnalysis, DifficultyLevel, SolvingTechnique, BOARD_SIZE};
use crate::validator::has_unique_solution;
use rand::seq::SliceRandom;
use rand::{thread_rng, Rng};

#[cfg(target_arch = "wasm32")]
use web_sys;

/*──────────────── CONFIG ────────────────*/

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
            max_attempts: 2_000, // was 1 000
            min_clues: 17,
            max_clues: 35,
            prefer_symmetry: true,
        }
    }
}

/*──────────────── GENERATOR ────────────────*/

pub struct PuzzleGenerator {
    config: GeneratorConfig,
}

impl PuzzleGenerator {
    pub fn new(config: GeneratorConfig) -> Self {
        Self { config }
    }

    pub fn with_difficulty(difficulty: DifficultyLevel) -> Self {
        let mut cfg = GeneratorConfig::default();
        cfg.target_difficulty = difficulty;

        match difficulty {
            DifficultyLevel::Easy => {
                cfg.min_clues = 35;
                cfg.max_clues = 45;
            }
            DifficultyLevel::Medium => {
                cfg.min_clues = 28;
                cfg.max_clues = 35;
            }
            DifficultyLevel::Hard => {
                cfg.min_clues = 22;
                cfg.max_clues = 28;
            }
            DifficultyLevel::Expert => {
                cfg.min_clues = 17;
                cfg.max_clues = 25;
            }
        }
        Self::new(cfg)
    }

    pub fn generate(&self) -> Option<Vec<Option<u8>>> {
        for attempt in 0..self.config.max_attempts {
            if let Some(puz) = self.generate_attempt() {
                if self.validate_puzzle(&puz) {
                    return Some(puz);
                }
            }
            #[cfg(target_arch = "wasm32")]
            if attempt % 100 == 0 && attempt > 0 {
                web_sys::console::log_1(&format!("Generation attempt {}", attempt).into());
            }
        }
        None
    }

    /*──────── STEP 1: full solution ────────*/

    fn generate_attempt(&self) -> Option<Vec<Option<u8>>> {
        let sol = self.generate_complete_solution()?;
        self.create_puzzle_from_solution(&sol)
    }

    fn generate_complete_solution(&self) -> Option<Vec<Option<u8>>> {
        let mut board = [0u8; BOARD_SIZE];
        let (mut row_m, mut col_m, mut box_m) = ([0u16; 9], [0u16; 9], [0u16; 9]);
        let mut rng = thread_rng();

        if self.fill_board_fast(&mut board, &mut row_m, &mut col_m, &mut box_m, 0, &mut rng) {
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

    fn fill_board_fast(
        &self,
        b: &mut [u8; BOARD_SIZE],
        row_m: &mut [u16; 9],
        col_m: &mut [u16; 9],
        box_m: &mut [u16; 9],
        idx: usize,
        rng: &mut impl Rng,
    ) -> bool {
        if idx == BOARD_SIZE {
            return true;
        }
        let (r, c) = (idx / 9, idx % 9);
        let bx = (r / 3) * 3 + (c / 3);

        if b[idx] != 0 {
            return self.fill_board_fast(b, row_m, col_m, box_m, idx + 1, rng);
        }

        let mut digits = [1u8, 2, 3, 4, 5, 6, 7, 8, 9];
        digits.shuffle(rng);

        for &d in &digits {
            let bit = 1u16 << (d - 1); // FIX: correct bit position
            if row_m[r] & bit == 0 && col_m[c] & bit == 0 && box_m[bx] & bit == 0 {
                b[idx] = d;
                row_m[r] |= bit;
                col_m[c] |= bit;
                box_m[bx] |= bit;

                if self.fill_board_fast(b, row_m, col_m, box_m, idx + 1, rng) {
                    return true;
                }

                b[idx] = 0;
                row_m[r] &= !bit;
                col_m[c] &= !bit;
                box_m[bx] &= !bit;
            }
        }
        false
    }

    /*──────── STEP 2: clue removal ────────*/

    /// Creates a puzzle by strategically removing clues from a complete solution
    fn create_puzzle_from_solution(&self, solution: &[Option<u8>]) -> Option<Vec<Option<u8>>> {
        let mut puzzle = solution.to_vec();
        let order = self.get_removal_order();
        let mut removals_since_last_uniq = 0;

        for &idx in &order {
            let saved = puzzle[idx];
            puzzle[idx] = None;

            // --- NEW: never block removal just because we are above max_clues.
            // Only protect the **lower bound** so we don’t dip below min_clues. ---
            let clues = puzzle.iter().filter(|c| c.is_some()).count();
            if clues < self.config.min_clues {
                puzzle[idx] = saved;
                break; // can’t remove more – would underflow
            }

            /* uniqueness throttling unchanged */
            let need_uniq = removals_since_last_uniq >= 3 || clues <= self.config.min_clues + 2;

            if need_uniq && !has_unique_solution(&puzzle) {
                puzzle[idx] = saved;
                removals_since_last_uniq = 0;
                continue;
            }
            removals_since_last_uniq = if need_uniq {
                0
            } else {
                removals_since_last_uniq + 1
            };

            // -------------------------------------------------------------------
            // SUCCESS CRITERIA:
            //  • hardest technique is in target band
            //  • AND we are now ≤ max_clues
            // -------------------------------------------------------------------
            if clues <= self.config.max_clues {
                let analysis = analyze_difficulty(&puzzle);
                if self.difficulty_matches_target(&analysis) {
                    return Some(puzzle);
                }
            }
        }

        // Fallback (rare): validate final puzzle
        if self.validate_puzzle(&puzzle) {
            Some(puzzle)
        } else {
            None
        }
    }

    /*──────── helpers ────────*/

    fn get_removal_order(&self) -> Vec<usize> {
        let mut rng = thread_rng();
        let mut indices: Vec<usize> = (0..BOARD_SIZE).collect();

        if self.config.prefer_symmetry {
            let mut pairs = Vec::<(usize, usize)>::new();
            let mut seen = vec![false; BOARD_SIZE];
            for i in 0..BOARD_SIZE {
                if seen[i] {
                    continue;
                }
                let s = self.get_symmetric_index(i);
                if s != i && !seen[s] {
                    pairs.push((i, s));
                    seen[i] = true;
                    seen[s] = true;
                } else {
                    pairs.push((i, i));
                    seen[i] = true;
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

    fn get_symmetric_index(&self, index: usize) -> usize {
        let row = index / 9;
        let col = index % 9;
        (8 - row) * 9 + (8 - col)
    }

    fn meets_clue_constraints(&self, p: &[Option<u8>]) -> bool {
        let n = p.iter().filter(|c| c.is_some()).count();
        n >= self.config.min_clues && n <= self.config.max_clues
    }

    fn validate_puzzle(&self, p: &[Option<u8>]) -> bool {
        has_unique_solution(p)
            && self.meets_clue_constraints(p)
            && self.difficulty_matches_target(&analyze_difficulty(p))
    }

    /// **Relaxed difficulty bands** – removed strict diversity caps
    fn difficulty_matches_target(&self, a: &DifficultyAnalysis) -> bool {
        use SolvingTechnique::*;
        match self.config.target_difficulty {
            DifficultyLevel::Easy => a.hardest_technique <= HiddenSingle,
            DifficultyLevel::Medium => a.hardest_technique <= BoxLineReduction,
            DifficultyLevel::Hard => a.hardest_technique <= XWing,
            DifficultyLevel::Expert => a.hardest_technique >= Swordfish,
        }
    }

    /*── legacy stubs kept for signature compatibility ──*/

    #[allow(dead_code)]
    fn fill_board_randomly(&self, _: &mut Vec<Option<u8>>, _: usize) -> bool {
        unreachable!("legacy stub")
    }
    #[allow(dead_code)]
    fn is_valid_placement(&self, board: &[Option<u8>], idx: usize, num: u8) -> bool {
        let r = idx / 9;
        let c = idx % 9;
        if (0..9).any(|i| board[r * 9 + i] == Some(num)) {
            return false;
        }
        if (0..9).any(|i| board[i * 9 + c] == Some(num)) {
            return false;
        }
        let br = (r / 3) * 3;
        let bc = (c / 3) * 3;
        for rr in br..br + 3 {
            for cc in bc..bc + 3 {
                if board[rr * 9 + cc] == Some(num) {
                    return false;
                }
            }
        }
        true
    }
}

/*──────── free helpers – signatures unchanged ────────*/

pub fn generate_puzzle(d: DifficultyLevel) -> Option<Vec<Option<u8>>> {
    PuzzleGenerator::with_difficulty(d).generate()
}

pub fn generate_multiple_puzzles(d: DifficultyLevel, count: usize) -> Vec<Vec<Option<u8>>> {
    let gen = PuzzleGenerator::with_difficulty(d);
    (0..count).filter_map(|_| gen.generate()).collect()
}

/*──────── tests (unchanged) ────────*/

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
        let empty = vec![None; BOARD_SIZE];
        let forty = (0..40)
            .map(|_| Some(1))
            .chain(std::iter::repeat(None).take(BOARD_SIZE - 40))
            .collect::<Vec<_>>();
        assert!(!g.meets_clue_constraints(&empty));
        assert!(g.meets_clue_constraints(&forty));
    }
}
