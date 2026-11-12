use crate::game::{
    BOARD_COLS, BOARD_ROWS, Coord, GameState, Player, WIN_STREAK, WINNING_LINES,
};

pub struct AI;

impl AI {
    pub fn choose_move(game: &GameState) -> Option<Coord> {
        let grid = game.grid();
        let cpu = Player::CPU;
        let user = Player::User;

        // 1. Immediate Win Check: Can CPU win in one move?
        if let Some(mv) = Self::find_immediate_win(grid, cpu) {
            return Some(mv);
        }

        // 2. Immediate Block Check: Can User win on their next move?
        if let Some(mv) = Self::find_immediate_win(grid, user) {
            return Some(mv);
        }

        // 3. Find candidate moves (empty cells adjacent to existing pieces or center)
        let candidates = Self::get_candidate_moves(grid);
        if candidates.is_empty() {
            return None;
        }

        // 4. Alpha-Beta minimax search (depth 2 for swift responsiveness + strategic foresight)
        let mut best_score = i32::MIN;
        let mut best_moves: Vec<Coord> = Vec::new();

        for &(r, c) in &candidates {
            let mut sim_grid = *grid;
            sim_grid[r][c] = Some(cpu);

            // Minimax evaluation for user response
            let score = Self::minimax(&sim_grid, 2, i32::MIN, i32::MAX, false);

            if score > best_score {
                best_score = score;
                best_moves.clear();
                best_moves.push((r, c));
            } else if score == best_score {
                best_moves.push((r, c));
            }
        }

        // Pick randomly among best scoring moves for variety
        if !best_moves.is_empty() {
            let pick_idx = fastrand::usize(..best_moves.len());
            return Some(best_moves[pick_idx]);
        }

        Some(candidates[0])
    }

    /// Checks if a player has an immediate 4-in-a-row opportunity
    fn find_immediate_win(
        grid: &[[Option<Player>; BOARD_COLS]; BOARD_ROWS],
        player: Player,
    ) -> Option<Coord> {
        for line in &WINNING_LINES {
            let mut player_count = 0;
            let mut empty_coord = None;

            for &(r, c) in line {
                match grid[r][c] {
                    Some(p) if p == player => player_count += 1,
                    None => empty_coord = Some((r, c)),
                    _ => {
                        // Occupied by opponent
                        player_count = 0;
                        break;
                    }
                }
            }

            if player_count == WIN_STREAK - 1 && empty_coord.is_some() {
                return empty_coord;
            }
        }
        None
    }

    /// Generates candidate cells that are empty and close to existing pieces (or board center)
    fn get_candidate_moves(grid: &[[Option<Player>; BOARD_COLS]; BOARD_ROWS]) -> Vec<Coord> {
        let mut candidates = Vec::new();
        let mut has_pieces = false;

        let center_r = BOARD_ROWS / 2;
        let center_c = BOARD_COLS / 2;

        for r in 0..BOARD_ROWS {
            for c in 0..BOARD_COLS {
                if grid[r][c].is_some() {
                    has_pieces = true;
                    continue;
                }

                // Check if within distance 1 of any placed piece
                let is_near = Self::is_near_piece(grid, r, c);
                if is_near {
                    candidates.push((r, c));
                }
            }
        }

        if !has_pieces {
            // First move: pick center
            return vec![(center_r, center_c)];
        }

        if candidates.is_empty() {
            // Fallback: any empty cell
            for r in 0..BOARD_ROWS {
                for c in 0..BOARD_COLS {
                    if grid[r][c].is_none() {
                        candidates.push((r, c));
                    }
                }
            }
        }

        // Sort candidates by proximity to center
        candidates.sort_by_key(|&(r, c)| {
            let dr = (r as isize - center_r as isize).abs();
            let dc = (c as isize - center_c as isize).abs();
            dr.max(dc)
        });

        candidates
    }

    fn is_near_piece(
        grid: &[[Option<Player>; BOARD_COLS]; BOARD_ROWS],
        r: usize,
        c: usize,
    ) -> bool {
        let min_r = r.saturating_sub(1);
        let max_r = (r + 1).min(BOARD_ROWS - 1);
        let min_c = c.saturating_sub(1);
        let max_c = (c + 1).min(BOARD_COLS - 1);

        for nr in min_r..=max_r {
            for nc in min_c..=max_c {
                if (nr != r || nc != c) && grid[nr][nc].is_some() {
                    return true;
                }
            }
        }
        false
    }

    fn minimax(
        grid: &[[Option<Player>; BOARD_COLS]; BOARD_ROWS],
        depth: usize,
        mut alpha: i32,
        mut beta: i32,
        is_maximizing: bool,
    ) -> i32 {
        // Check terminal states
        if let Some(winner) = Self::check_grid_winner(grid) {
            return match winner {
                Player::CPU => 100_000 + depth as i32,
                Player::User => -100_000 - depth as i32,
            };
        }

        if depth == 0 {
            return Self::evaluate_board(grid);
        }

        let candidates = Self::get_candidate_moves(grid);
        if candidates.is_empty() {
            return 0; // Draw
        }

        if is_maximizing {
            let mut max_eval = i32::MIN;
            for &(r, c) in candidates.iter().take(12) {
                let mut next_grid = *grid;
                next_grid[r][c] = Some(Player::CPU);
                let eval = Self::minimax(&next_grid, depth - 1, alpha, beta, false);
                max_eval = max_eval.max(eval);
                alpha = alpha.max(eval);
                if beta <= alpha {
                    break;
                }
            }
            max_eval
        } else {
            let mut min_eval = i32::MAX;
            for &(r, c) in candidates.iter().take(12) {
                let mut next_grid = *grid;
                next_grid[r][c] = Some(Player::User);
                let eval = Self::minimax(&next_grid, depth - 1, alpha, beta, true);
                min_eval = min_eval.min(eval);
                beta = beta.min(eval);
                if beta <= alpha {
                    break;
                }
            }
            min_eval
        }
    }

    fn check_grid_winner(
        grid: &[[Option<Player>; BOARD_COLS]; BOARD_ROWS],
    ) -> Option<Player> {
        for line in &WINNING_LINES {
            let first = grid[line[0].0][line[0].1];
            if let Some(player) = first {
                if line[1..].iter().all(|&(r, c)| grid[r][c] == Some(player)) {
                    return Some(player);
                }
            }
        }
        None
    }

    fn evaluate_board(grid: &[[Option<Player>; BOARD_COLS]; BOARD_ROWS]) -> i32 {
        let mut score = 0;
        let center_r = BOARD_ROWS / 2;
        let center_c = BOARD_COLS / 2;

        for line in &WINNING_LINES {
            let mut cpu_count = 0;
            let mut user_count = 0;

            for &(r, c) in line {
                match grid[r][c] {
                    Some(Player::CPU) => cpu_count += 1,
                    Some(Player::User) => user_count += 1,
                    None => {}
                }
            }

            if cpu_count > 0 && user_count > 0 {
                continue; // Blocked line
            }

            if cpu_count > 0 {
                score += match cpu_count {
                    3 => 800,
                    2 => 60,
                    1 => 8,
                    _ => 0,
                };
            } else if user_count > 0 {
                score -= match user_count {
                    3 => 1500, // Urgent to prevent user 3-in-a-row
                    2 => 100,
                    1 => 10,
                    _ => 0,
                };
            }
        }

        // Positional bonus: center control
        for r in 0..BOARD_ROWS {
            for c in 0..BOARD_COLS {
                if let Some(player) = grid[r][c] {
                    let dist = (r as isize - center_r as isize).abs()
                        + (c as isize - center_c as isize).abs();
                    let pos_bonus = (6 - dist).max(0) as i32 * 2;
                    if player == Player::CPU {
                        score += pos_bonus;
                    } else {
                        score -= pos_bonus;
                    }
                }
            }
        }

        score
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ai_takes_immediate_win() {
        let mut game = GameState::new();
        // CPU has 3 pieces at (2, 1), (2, 2), (2, 3). Open spot is (2, 4) or (2, 0)
        // User moves on row 0
        let _ = game.make_move(0, 0); // User
        let _ = game.make_move(2, 1); // CPU
        let _ = game.make_move(0, 1); // User
        let _ = game.make_move(2, 2); // CPU
        let _ = game.make_move(0, 2); // User
        let _ = game.make_move(2, 3); // CPU
        let _ = game.make_move(5, 5); // User

        // Now CPU's turn. It should pick (2, 0) or (2, 4) to win immediately
        let mv = AI::choose_move(&game).expect("AI should choose a move");
        assert!(mv == (2, 0) || mv == (2, 4), "Expected (2,0) or (2,4), got {:?}", mv);
    }

    #[test]
    fn test_ai_blocks_user_win() {
        let mut game = GameState::new();
        // User has 3 pieces at (3, 1), (3, 2), (3, 3). Open spots are (3, 0) or (3, 4)
        let _ = game.make_move(3, 1); // User
        let _ = game.make_move(0, 0); // CPU
        let _ = game.make_move(3, 2); // User
        let _ = game.make_move(0, 1); // CPU
        let _ = game.make_move(3, 3); // User

        // CPU turn: must block at (3, 0) or (3, 4)
        let mv = AI::choose_move(&game).expect("AI should choose a move");
        assert!(mv == (3, 0) || mv == (3, 4), "AI must block user at (3,0) or (3,4), got {:?}", mv);
    }

    #[test]
    fn test_ai_first_move_center() {
        let mut game = GameState::new();
        let _ = game.make_move(0, 0); // User starts in corner
        let mv = AI::choose_move(&game).expect("AI should choose a move");
        // Center or near center
        let dist_to_center = (mv.0 as isize - 3).abs() + (mv.1 as isize - 3).abs();
        assert!(dist_to_center <= 4, "AI should play near center");
    }
}
