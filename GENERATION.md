# Puzzle Generation Details

This file documents the generator components and the exact constraints used per difficulty. Function names are given with their file locations.

## Configuration (all in `src/generator.rs`)
- **`GeneratorConfig::for_difficulty`** sets difficulty targets:
  - VeryEasy: min clues 44, max 52, branching factor 1.0–1.7, target 1.4, max_attempts=3_000, unique_check_interval=2.
  - Easy: min 40, max 46, branching factor 1.3–2.0, target 1.6, max_attempts=3_000, unique_check_interval=3.
  - Medium: min 32, max 38, branching factor 2.0–2.7, target 2.3, max_attempts=3_000, unique_check_interval=4.
  - Hard: min 25, max 30, branching factor 3.5–4.5, target 3.8, max_attempts=2_500, unique_check_interval=5.
  - Expert: min 17, max 24, branching factor 4.5–6.0, target 5.0, max_attempts=4_000, unique_check_interval=6.
- **Progress hooks** (`report_progress_with_meta`, `search_progress`): emit stage, percent, and metadata (attempt number, max attempts, best score, best clue count).

## Entry points
- **WASM API:** `createGameWithSeed` and `create_puzzle_with_seed` in `src/wasm_exports.rs`.
- **Rust API:** `generate_puzzle` / `generate_puzzle_enhanced` / `generate_puzzle_with_branching_factor` in `src/generator.rs`.
- **Generator struct:** `PuzzleGenerator::generate` drives the attempt loop up to `max_attempts`.

## Per-attempt flow
- **`generate_attempt`**:
  1) Build a full solved grid via `generate_complete_solution` → `fill_board_fast` (bitmask backtracking).
  2) Carve puzzle via `create_puzzle_with_branching_factor_control` (or seeded equivalent).
- **`create_puzzle_with_branching_factor_control`** (core carving loop):
  - Removal order from `get_removal_order` (prefers symmetry).
  - Tracks clue count incrementally while setting cells to `None`.
  - Every few removals: report progress (capped to ~60% during carving) and sample uniqueness:
    - Uniqueness check if `since_unique_check >= unique_check_interval` or close to min clues; uses `has_unique_solution` (`src/validator.rs`).
  - Skip expensive scoring on every removal: if not near min clues and step % 3 ≠ 0, continue.
  - When sampling:
    - Compute branching factor: `calculate_branching_factor` on a `HumanStyleSolver` (`src/solver.rs`).
    - Analyze difficulty: `analyze_difficulty` (`src/difficulty.rs`) to find hardest technique/branching factor.
    - Evaluate constraints: `meets_all_constraints` (clue bounds, branching factor window, difficulty target) and `difficulty_matches_target`; `difficulty_overshoot` prunes over-hard puzzles early.
    - Score: `bf_diff + (clues - min_clues)*0.1`; track best candidate and report progress with metadata (attempt, best score, best clue count).
    - Early exit if branching factor is within half the tolerance and constraints pass.
- **Seeded path** `create_puzzle_with_seed` (in `src/wasm_exports.rs`) mirrors the above with deterministic RNG per seed.

## Constraints and validation helpers
- **`meets_all_constraints`**: enforces clue count within min/max, hardest technique within allowed range per difficulty, and branching factor within min/max.
- **`difficulty_matches_target`**: caps allowed hardest technique per level (e.g., Hard must be between XWing and Swordfish; Expert must be ≥ XYWing).
- **`difficulty_overshoot`**: stops carving when the puzzle is already harder than the target band.
- **Uniqueness**: `has_unique_solution` (`src/validator.rs`) counts solutions with backtracking; used both during carving (interval checks) and before accepting a candidate.

## Progress semantics
- Stages:
  - “Starting generator” → solution build.
  - “Carving clues” → up to ~60%.
  - “Evaluating candidate puzzles” → scales with attempt/max_attempts into the 90s, includes metadata.
  - “Validating uniqueness” / “Analyzing difficulty” / “Finalizing puzzle” → final increments to 100%.
- Metadata delivered to JS/worker/UI:
  - `attempt`: zero-based attempt index.
  - `max_attempts`: configured cap for the difficulty.
  - `best_score`: current best score (lower is better).
  - `best_clue_count`: clue count of best candidate so far.

## Why evaluation time varies
- Sparse (Expert) boards make uniqueness counting and difficulty analysis much slower per attempt.
- Some attempts fail quickly (early uniqueness fail); others reach scoring and run the solver/analysis, which is expensive.
- Lowering `max_attempts` speeds completion but can increase variability in the final puzzle’s exact branching-factor/difficulty alignment. Early-exit still returns the best candidate found if constraints aren’t perfectly met.
