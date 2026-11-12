use std::cell::RefCell;
use std::rc::Rc;
use std::time::Duration;

use slint::{ComponentHandle, Global, Model, ModelRc, Timer, TimerMode, VecModel};

use crate::ai::AI;
use crate::game::{
    BOARD_COLS, BOARD_ROWS, GameState, GameStatus, Player,
};
use crate::history::{DEFAULT_HISTORY_PATH, HistoryManager, MatchResult};
use crate::{
    App, State, TBall, TBoard, TFinish, TScreen, TSlot, TStat,
};

pub struct GameController {
    inner: Rc<RefCell<GameControllerInner>>,
}

struct GameControllerInner {
    game: GameState,
    history: HistoryManager,
    rows: Vec<Rc<VecModel<TSlot>>>,
    app_weak: slint::Weak<App>,
    cpu_timer: Option<Timer>,
}

impl GameController {
    pub fn new(app: &App) -> Self {
        // Initialize empty 7x7 board models
        let rows: Vec<Rc<VecModel<TSlot>>> = (0..BOARD_ROWS)
            .map(|_| {
                let row_slots: Vec<TSlot> = (0..BOARD_COLS)
                    .map(|_| TSlot {
                        ball: TBall::None,
                        active: false,
                    })
                    .collect();
                Rc::new(VecModel::from(row_slots))
            })
            .collect();

        let grid_vec: Vec<ModelRc<TSlot>> = rows
            .iter()
            .map(|r| ModelRc::from(r.clone()))
            .collect();

        let tboard = TBoard {
            board: ModelRc::new(VecModel::from(grid_vec)),
        };

        let state = State::get(app);
        state.set_board(tboard);
        state.set_turn(TBall::User);
        state.set_last_move((-1, -1));
        state.set_finish(TFinish::None);
        state.set_step_count(0);
        state.set_screen(TScreen::Board);

        let history = HistoryManager::new(DEFAULT_HISTORY_PATH);
        state.set_stat(Self::build_tstat(&history));

        let inner = Rc::new(RefCell::new(GameControllerInner {
            game: GameState::new(),
            history,
            rows,
            app_weak: app.as_weak(),
            cpu_timer: None,
        }));

        let controller = Self { inner };
        controller.setup_callbacks(app);
        controller
    }

    fn build_tstat(history: &HistoryManager) -> TStat {
        TStat {
            r#match: history.total_matches() as i32,
            r#finished: history.finished() as i32,
            r#unfinished: history.unfinished() as i32,
            r#win: history.wins() as i32,
            r#tie: history.ties() as i32,
            r#lost: history.losses() as i32,
        }
    }

    fn setup_callbacks(&self, app: &App) {
        let state = State::get(app);

        // Cell Clicked Callback
        {
            let inner_clone = self.inner.clone();
            state.on_cell_clicked(move |rin, cin| {
                if rin < 0 || cin < 0 {
                    return;
                }
                Self::handle_user_click(inner_clone.clone(), rin as usize, cin as usize);
            });
        }

        // Reset Board Callback
        {
            let inner_clone = self.inner.clone();
            state.on_reset_board(move || {
                Self::handle_reset(inner_clone.clone());
            });
        }
    }

    fn handle_user_click(inner_rc: Rc<RefCell<GameControllerInner>>, row: usize, col: usize) {
        let mut inner = inner_rc.borrow_mut();

        // Must be user turn, cell valid, and game in progress
        if inner.game.current_turn() != Player::User || !inner.game.is_valid_move(row, col) {
            return;
        }

        // Execute User move
        let status = match inner.game.make_move(row, col) {
            Ok(s) => s,
            Err(_) => return,
        };

        // Update UI board cell
        inner.rows[row].set_row_data(
            col,
            TSlot {
                ball: TBall::User,
                active: false,
            },
        );

        let Some(app) = inner.app_weak.upgrade() else {
            return;
        };
        let state = State::get(&app);
        // Slint maps `{ row: int, col: int }` to `(col, row)` alphabetically
        state.set_last_move((col as i32, row as i32));
        state.set_step_count(inner.game.total_steps() as i32);

        match status {
            GameStatus::Won {
                winning_line, ..
            } => {
                // Highlight winning line
                for &(wr, wc) in &winning_line {
                    inner.rows[wr].set_row_data(
                        wc,
                        TSlot {
                            ball: TBall::User,
                            active: true,
                        },
                    );
                }
                let total_steps = inner.game.total_steps();
                let user_steps = inner.game.user_steps();
                state.set_finish(TFinish::Win);
                inner.history.record_match(
                    MatchResult::Win,
                    total_steps,
                    user_steps,
                );
                state.set_stat(Self::build_tstat(&inner.history));
            }
            GameStatus::Tie => {
                let total_steps = inner.game.total_steps();
                let user_steps = inner.game.user_steps();
                state.set_finish(TFinish::Tie);
                inner.history.record_match(
                    MatchResult::Tie,
                    total_steps,
                    user_steps,
                );
                state.set_stat(Self::build_tstat(&inner.history));
            }
            GameStatus::InProgress => {
                state.set_turn(TBall::CPU);

                // Schedule CPU move after a natural 300ms delay
                let inner_weak = Rc::downgrade(&inner_rc);
                let timer = Timer::default();
                timer.start(
                    TimerMode::SingleShot,
                    Duration::from_millis(300),
                    move || {
                        if let Some(inner_upgrade) = inner_weak.upgrade() {
                            Self::handle_cpu_move(inner_upgrade);
                        }
                    },
                );
                inner.cpu_timer = Some(timer);
            }
        }
    }

    fn handle_cpu_move(inner_rc: Rc<RefCell<GameControllerInner>>) {
        let mut inner = inner_rc.borrow_mut();
        inner.cpu_timer = None;

        if inner.game.current_turn() != Player::CPU
            || *inner.game.status() != GameStatus::InProgress
        {
            return;
        }

        let Some((cr, cc)) = AI::choose_move(&inner.game) else {
            return;
        };

        let status = match inner.game.make_move(cr, cc) {
            Ok(s) => s,
            Err(_) => return,
        };

        // Update UI board cell for CPU
        inner.rows[cr].set_row_data(
            cc,
            TSlot {
                ball: TBall::CPU,
                active: false,
            },
        );

        let Some(app) = inner.app_weak.upgrade() else {
            return;
        };
        let state = State::get(&app);
        // Slint maps `{ row: int, col: int }` to `(col, row)` alphabetically
        state.set_last_move((cc as i32, cr as i32));
        state.set_step_count(inner.game.total_steps() as i32);

        match status {
            GameStatus::Won {
                winning_line, ..
            } => {
                // Highlight winning line
                for &(wr, wc) in &winning_line {
                    inner.rows[wr].set_row_data(
                        wc,
                        TSlot {
                            ball: TBall::CPU,
                            active: true,
                        },
                    );
                }
                let total_steps = inner.game.total_steps();
                let user_steps = inner.game.user_steps();
                state.set_finish(TFinish::Lost);
                inner.history.record_match(
                    MatchResult::Lost,
                    total_steps,
                    user_steps,
                );
                state.set_stat(Self::build_tstat(&inner.history));
            }
            GameStatus::Tie => {
                let total_steps = inner.game.total_steps();
                let user_steps = inner.game.user_steps();
                state.set_finish(TFinish::Tie);
                inner.history.record_match(
                    MatchResult::Tie,
                    total_steps,
                    user_steps,
                );
                state.set_stat(Self::build_tstat(&inner.history));
            }
            GameStatus::InProgress => {
                state.set_turn(TBall::User);
            }
        }
    }

    fn handle_reset(inner_rc: Rc<RefCell<GameControllerInner>>) {
        let mut inner = inner_rc.borrow_mut();
        inner.cpu_timer = None;

        // If game had started and was not finished, record as Unfinished
        if inner.game.has_started() && *inner.game.status() == GameStatus::InProgress {
            let total_steps = inner.game.total_steps();
            let user_steps = inner.game.user_steps();
            inner.history.record_match(
                MatchResult::Unfinished,
                total_steps,
                user_steps,
            );
        }

        // Reset backend game state
        inner.game.reset();

        // Clear all 49 slots in UI board
        for r in 0..BOARD_ROWS {
            for c in 0..BOARD_COLS {
                inner.rows[r].set_row_data(
                    c,
                    TSlot {
                        ball: TBall::None,
                        active: false,
                    },
                );
            }
        }

        let Some(app) = inner.app_weak.upgrade() else {
            return;
        };
        let state = State::get(&app);
        state.set_last_move((-1, -1));
        state.set_finish(TFinish::None);
        state.set_turn(TBall::User);
        state.set_step_count(0);
        state.set_stat(Self::build_tstat(&inner.history));
    }
}
