pub const BOARD_ROWS: usize = 7;
pub const BOARD_COLS: usize = 7;
pub const WIN_STREAK: usize = 4;
pub const TOTAL_CELLS: usize = BOARD_ROWS * BOARD_COLS;
pub const TOTAL_WINNING_LINES: usize = 88;

pub type Coord = (usize, usize);
pub type WinningLine = [Coord; WIN_STREAK];

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum Player {
    User,
    CPU,
}

impl Player {
    pub fn opponent(&self) -> Self {
        match self {
            Player::User => Player::CPU,
            Player::CPU => Player::User,
        }
    }
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub enum GameStatus {
    InProgress,
    Won {
        winner: Player,
        winning_line: WinningLine,
    },
    Tie,
}

pub const fn generate_winning_lines() -> [WinningLine; TOTAL_WINNING_LINES] {
    let mut lines = [[(0, 0); WIN_STREAK]; TOTAL_WINNING_LINES];
    let mut idx = 0;

    // 1. Horizontal lines (28)
    let mut r = 0;
    while r < BOARD_ROWS {
        let mut c = 0;
        while c <= BOARD_COLS - WIN_STREAK {
            lines[idx] = [(r, c), (r, c + 1), (r, c + 2), (r, c + 3)];
            idx += 1;
            c += 1;
        }
        r += 1;
    }

    // 2. Vertical lines (28)
    let mut r = 0;
    while r <= BOARD_ROWS - WIN_STREAK {
        let mut c = 0;
        while c < BOARD_COLS {
            lines[idx] = [(r, c), (r + 1, c), (r + 2, c), (r + 3, c)];
            idx += 1;
            c += 1;
        }
        r += 1;
    }

    // 3. Diagonal down-right lines (16)
    let mut r = 0;
    while r <= BOARD_ROWS - WIN_STREAK {
        let mut c = 0;
        while c <= BOARD_COLS - WIN_STREAK {
            lines[idx] = [
                (r, c),
                (r + 1, c + 1),
                (r + 2, c + 2),
                (r + 3, c + 3),
            ];
            idx += 1;
            c += 1;
        }
        r += 1;
    }

    // 4. Diagonal up-right lines (16)
    let mut r = WIN_STREAK - 1;
    while r < BOARD_ROWS {
        let mut c = 0;
        while c <= BOARD_COLS - WIN_STREAK {
            lines[idx] = [
                (r, c),
                (r - 1, c + 1),
                (r - 2, c + 2),
                (r - 3, c + 3),
            ];
            idx += 1;
            c += 1;
        }
        r += 1;
    }

    lines
}

pub static WINNING_LINES: [WinningLine; TOTAL_WINNING_LINES] = generate_winning_lines();

#[derive(Clone, Debug)]
pub struct GameState {
    grid: [[Option<Player>; BOARD_COLS]; BOARD_ROWS],
    current_turn: Player,
    status: GameStatus,
    total_steps: u32,
    user_steps: u32,
    last_move: Option<Coord>,
}

impl Default for GameState {
    fn default() -> Self {
        Self::new()
    }
}

impl GameState {
    pub fn new() -> Self {
        Self {
            grid: [[None; BOARD_COLS]; BOARD_ROWS],
            current_turn: Player::User,
            status: GameStatus::InProgress,
            total_steps: 0,
            user_steps: 0,
            last_move: None,
        }
    }

    pub fn reset(&mut self) {
        self.grid = [[None; BOARD_COLS]; BOARD_ROWS];
        self.current_turn = Player::User;
        self.status = GameStatus::InProgress;
        self.total_steps = 0;
        self.user_steps = 0;
        self.last_move = None;
    }

    pub fn is_valid_move(&self, row: usize, col: usize) -> bool {
        if self.status != GameStatus::InProgress {
            return false;
        }
        if row >= BOARD_ROWS || col >= BOARD_COLS {
            return false;
        }
        self.grid[row][col].is_none()
    }

    pub fn make_move(&mut self, row: usize, col: usize) -> Result<GameStatus, &'static str> {
        if self.status != GameStatus::InProgress {
            return Err("Game is already finished");
        }
        if row >= BOARD_ROWS || col >= BOARD_COLS {
            return Err("Move is out of board bounds");
        }
        if self.grid[row][col].is_some() {
            return Err("Cell is already occupied");
        }

        let player = self.current_turn;
        self.grid[row][col] = Some(player);
        self.total_steps += 1;
        if player == Player::User {
            self.user_steps += 1;
        }
        self.last_move = Some((row, col));

        // Check for win
        if let Some(line) = self.check_win_at(row, col, player) {
            self.status = GameStatus::Won {
                winner: player,
                winning_line: line,
            };
            return Ok(self.status.clone());
        }

        // Check for tie (board full)
        if self.total_steps as usize >= TOTAL_CELLS {
            self.status = GameStatus::Tie;
            return Ok(self.status.clone());
        }

        // Advance turn
        self.current_turn = player.opponent();
        Ok(self.status.clone())
    }

    pub fn check_win_at(&self, row: usize, col: usize, player: Player) -> Option<WinningLine> {
        for line in &WINNING_LINES {
            let contains_coord = line.iter().any(|&(r, c)| r == row && c == col);
            if contains_coord {
                let all_match = line.iter().all(|&(r, c)| self.grid[r][c] == Some(player));
                if all_match {
                    return Some(*line);
                }
            }
        }
        None
    }

    pub fn grid(&self) -> &[[Option<Player>; BOARD_COLS]; BOARD_ROWS] {
        &self.grid
    }

    pub fn current_turn(&self) -> Player {
        self.current_turn
    }

    pub fn status(&self) -> &GameStatus {
        &self.status
    }

    pub fn total_steps(&self) -> u32 {
        self.total_steps
    }

    pub fn user_steps(&self) -> u32 {
        self.user_steps
    }

    #[allow(dead_code)]
    pub fn last_move(&self) -> Option<Coord> {
        self.last_move
    }

    pub fn has_started(&self) -> bool {
        self.total_steps > 0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_winning_lines_count() {
        assert_eq!(WINNING_LINES.len(), 88);
    }

    #[test]
    fn test_horizontal_win() {
        let mut game = GameState::new();
        // User moves along row 0: (0, 0), (0, 1), (0, 2), (0, 3)
        // CPU moves along row 1: (1, 0), (1, 1), (1, 2)
        assert!(game.make_move(0, 0).is_ok()); // User
        assert!(game.make_move(1, 0).is_ok()); // CPU
        assert!(game.make_move(0, 1).is_ok()); // User
        assert!(game.make_move(1, 1).is_ok()); // CPU
        assert!(game.make_move(0, 2).is_ok()); // User
        assert!(game.make_move(1, 2).is_ok()); // CPU
        let res = game.make_move(0, 3); // User wins
        assert!(matches!(
            res,
            Ok(GameStatus::Won {
                winner: Player::User,
                winning_line: [(0, 0), (0, 1), (0, 2), (0, 3)]
            })
        ));
        assert_eq!(game.user_steps(), 4);
        assert_eq!(game.total_steps(), 7);
    }

    #[test]
    fn test_vertical_win() {
        let mut game = GameState::new();
        // User moves down col 3: (1, 3), (2, 3), (3, 3), (4, 3)
        assert!(game.make_move(1, 3).is_ok()); // User
        assert!(game.make_move(0, 0).is_ok()); // CPU
        assert!(game.make_move(2, 3).is_ok()); // User
        assert!(game.make_move(0, 1).is_ok()); // CPU
        assert!(game.make_move(3, 3).is_ok()); // User
        assert!(game.make_move(0, 2).is_ok()); // CPU
        let res = game.make_move(4, 3); // User wins
        assert!(matches!(
            res,
            Ok(GameStatus::Won {
                winner: Player::User,
                winning_line: [(1, 3), (2, 3), (3, 3), (4, 3)]
            })
        ));
    }

    #[test]
    fn test_diagonal_down_right_win() {
        let mut game = GameState::new();
        // User moves along diagonal (2, 2) to (5, 5)
        assert!(game.make_move(2, 2).is_ok()); // User
        assert!(game.make_move(0, 0).is_ok()); // CPU
        assert!(game.make_move(3, 3).is_ok()); // User
        assert!(game.make_move(0, 1).is_ok()); // CPU
        assert!(game.make_move(4, 4).is_ok()); // User
        assert!(game.make_move(0, 2).is_ok()); // CPU
        let res = game.make_move(5, 5); // User wins
        assert!(matches!(
            res,
            Ok(GameStatus::Won {
                winner: Player::User,
                winning_line: [(2, 2), (3, 3), (4, 4), (5, 5)]
            })
        ));
    }

    #[test]
    fn test_diagonal_up_right_win() {
        let mut game = GameState::new();
        // User moves along diagonal (3, 0), (2, 1), (1, 2), (0, 3)
        assert!(game.make_move(3, 0).is_ok()); // User
        assert!(game.make_move(6, 6).is_ok()); // CPU
        assert!(game.make_move(2, 1).is_ok()); // User
        assert!(game.make_move(6, 5).is_ok()); // CPU
        assert!(game.make_move(1, 2).is_ok()); // User
        assert!(game.make_move(6, 4).is_ok()); // CPU
        let res = game.make_move(0, 3); // User wins
        assert!(matches!(
            res,
            Ok(GameStatus::Won {
                winner: Player::User,
                winning_line: [(3, 0), (2, 1), (1, 2), (0, 3)]
            })
        ));
    }

    #[test]
    fn test_invalid_move_occupied() {
        let mut game = GameState::new();
        assert!(game.make_move(3, 3).is_ok());
        assert_eq!(game.make_move(3, 3), Err("Cell is already occupied"));
    }

    #[test]
    fn test_invalid_move_after_finish() {
        let mut game = GameState::new();
        let _ = game.make_move(0, 0);
        let _ = game.make_move(1, 0);
        let _ = game.make_move(0, 1);
        let _ = game.make_move(1, 1);
        let _ = game.make_move(0, 2);
        let _ = game.make_move(1, 2);
        let _ = game.make_move(0, 3); // Win
        assert_eq!(game.make_move(5, 5), Err("Game is already finished"));
    }
}
