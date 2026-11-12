use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

pub const DEFAULT_HISTORY_PATH: &str = "data/history.json";
pub const MAX_HISTORY_MATCHES: usize = 20;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub enum MatchResult {
    Win,
    Lost,
    Tie,
    Unfinished,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct MatchRecord {
    pub id: usize,
    pub timestamp: u64,
    pub result: MatchResult,
    pub total_steps: u32,
    pub user_steps: u32,
}

#[derive(Serialize, Deserialize, Clone, Debug, Default, PartialEq, Eq)]
pub struct HistoryData {
    pub matches: Vec<MatchRecord>,
    pub total_matches: usize,
    pub finished: usize,
    pub unfinished: usize,
    pub wins: usize,
    pub ties: usize,
    pub losses: usize,
}

pub struct HistoryManager {
    path: PathBuf,
    data: HistoryData,
}

impl HistoryManager {
    pub fn new<P: AsRef<Path>>(path: P) -> Self {
        let p = path.as_ref().to_path_buf();
        let data = Self::load_from_path(&p);
        Self { path: p, data }
    }

    fn load_from_path(path: &Path) -> HistoryData {
        if let Ok(content) = fs::read_to_string(path) {
            if let Ok(loaded) = serde_json::from_str::<HistoryData>(&content) {
                return loaded;
            }
        }
        HistoryData::default()
    }

    pub fn save(&self) -> Result<(), std::io::Error> {
        if let Some(parent) = self.path.parent() {
            if !parent.as_os_str().is_empty() {
                fs::create_dir_all(parent)?;
            }
        }
        let json = serde_json::to_string_pretty(&self.data)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;
        fs::write(&self.path, json)
    }

    pub fn record_match(
        &mut self,
        result: MatchResult,
        total_steps: u32,
        user_steps: u32,
    ) {
        self.data.total_matches += 1;
        match result {
            MatchResult::Win => {
                self.data.finished += 1;
                self.data.wins += 1;
            }
            MatchResult::Lost => {
                self.data.finished += 1;
                self.data.losses += 1;
            }
            MatchResult::Tie => {
                self.data.finished += 1;
                self.data.ties += 1;
            }
            MatchResult::Unfinished => {
                self.data.unfinished += 1;
            }
        }

        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);

        let record = MatchRecord {
            id: self.data.total_matches,
            timestamp,
            result,
            total_steps,
            user_steps,
        };

        self.data.matches.push(record);
        if self.data.matches.len() > MAX_HISTORY_MATCHES {
            self.data.matches.remove(0);
        }

        let _ = self.save();
    }

    #[allow(dead_code)]
    pub fn data(&self) -> &HistoryData {
        &self.data
    }

    pub fn total_matches(&self) -> usize {
        self.data.total_matches
    }

    pub fn finished(&self) -> usize {
        self.data.finished
    }

    pub fn unfinished(&self) -> usize {
        self.data.unfinished
    }

    pub fn wins(&self) -> usize {
        self.data.wins
    }

    pub fn ties(&self) -> usize {
        self.data.ties
    }

    pub fn losses(&self) -> usize {
        self.data.losses
    }

    #[allow(dead_code)]
    pub fn matches(&self) -> &[MatchRecord] {
        &self.data.matches
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_record_match_and_fifo() {
        let temp_dir = std::env::temp_dir().join(format!("test_rof4_hist_{}", fastrand::u64(..)));
        let path = temp_dir.join("history.json");

        let mut mgr = HistoryManager::new(&path);

        // Record 25 matches
        for i in 0..25 {
            let res = if i % 4 == 0 {
                MatchResult::Win
            } else if i % 4 == 1 {
                MatchResult::Lost
            } else if i % 4 == 2 {
                MatchResult::Tie
            } else {
                MatchResult::Unfinished
            };
            mgr.record_match(res, (i + 4) as u32, (i + 2) as u32);
        }

        assert_eq!(mgr.total_matches(), 25);
        assert_eq!(mgr.matches().len(), 20); // FIFO capped at 20
        assert_eq!(mgr.matches().first().unwrap().id, 6); // First 5 dropped
        assert_eq!(mgr.matches().last().unwrap().id, 25);

        // Reload from disk and verify persistence
        let mgr2 = HistoryManager::new(&path);
        assert_eq!(mgr2.total_matches(), 25);
        assert_eq!(mgr2.matches().len(), 20);
        assert_eq!(mgr2.wins(), mgr.wins());
        assert_eq!(mgr2.losses(), mgr.losses());
        assert_eq!(mgr2.ties(), mgr.ties());
        assert_eq!(mgr2.unfinished(), mgr.unfinished());

        let _ = fs::remove_dir_all(temp_dir);
    }
}
