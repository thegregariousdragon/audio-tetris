use rusqlite::{Connection, Result, params};
use std::sync::{Arc, Mutex};
use std::time::SystemTime;

use crate::logic::GameState;

#[allow(dead_code)]
#[derive(Clone, Debug)]
pub struct SaveSlotInfo {
    pub slot_id: usize,
    pub timestamp: String,
    pub score: u32,
    pub level: u32,
    pub lines: u32,
    pub difficulty: String,
}

#[allow(dead_code)]
#[derive(Clone, Debug)]
pub struct HighScoreEntry {
    pub id: i64,
    pub timestamp: String,
    pub score: u32,
    pub level: u32,
    pub lines: u32,
    pub difficulty: String,
}

#[derive(Clone, Debug, Default)]
pub struct PlayerStats {
    pub total_games_played: u32,
    pub total_lines_cleared: u32,
    pub high_score: u32,
}

pub struct Database {
    conn: Arc<Mutex<Connection>>,
}

impl Database {
    pub fn new(path: &str) -> Result<Self, rusqlite::Error> {
        let conn = Connection::open(path)?;
        let db = Self {
            conn: Arc::new(Mutex::new(conn)),
        };
        db.init_tables()?;
        Ok(db)
    }

    #[allow(dead_code)]
    pub fn new_in_memory() -> Result<Self, rusqlite::Error> {
        let conn = Connection::open_in_memory()?;
        let db = Self {
            conn: Arc::new(Mutex::new(conn)),
        };
        db.init_tables()?;
        Ok(db)
    }

    pub fn init_tables(&self) -> Result<(), rusqlite::Error> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "CREATE TABLE IF NOT EXISTS save_slots (
                slot_id INTEGER PRIMARY KEY,
                timestamp TEXT NOT NULL,
                score INTEGER NOT NULL,
                level INTEGER NOT NULL,
                lines INTEGER NOT NULL,
                difficulty TEXT NOT NULL,
                game_state_json TEXT NOT NULL
            )",
            [],
        )?;

        conn.execute(
            "CREATE TABLE IF NOT EXISTS high_scores (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                timestamp TEXT NOT NULL,
                score INTEGER NOT NULL,
                level INTEGER NOT NULL,
                lines INTEGER NOT NULL,
                difficulty TEXT NOT NULL
            )",
            [],
        )?;

        conn.execute(
            "CREATE TABLE IF NOT EXISTS player_stats (
                id INTEGER PRIMARY KEY CHECK (id = 1),
                total_games_played INTEGER DEFAULT 0,
                total_lines_cleared INTEGER DEFAULT 0,
                high_score INTEGER DEFAULT 0
            )",
            [],
        )?;

        conn.execute(
            "INSERT OR IGNORE INTO player_stats (id, total_games_played, total_lines_cleared, high_score)
             VALUES (1, 0, 0, 0)",
            [],
        )?;

        Ok(())
    }

    pub fn save_slot(
        &self,
        slot_id: usize,
        gs: &GameState,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let json = serde_json::to_string(gs)?;
        let secs = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);
        let days = secs / 86400;
        let hours = (secs % 86400) / 3600;
        let mins = (secs % 3600) / 60;
        let timestamp = format!("Day {} {:02}:{:02} UTC", days, hours, mins);
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT INTO save_slots (slot_id, timestamp, score, level, lines, difficulty, game_state_json)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)
             ON CONFLICT(slot_id) DO UPDATE SET
                timestamp = excluded.timestamp,
                score = excluded.score,
                level = excluded.level,
                lines = excluded.lines,
                difficulty = excluded.difficulty,
                game_state_json = excluded.game_state_json",
            params![
                slot_id as i64,
                timestamp,
                gs.score as i64,
                gs.level as i64,
                gs.total_lines as i64,
                gs.difficulty.as_str(),
                json
            ],
        )?;
        Ok(())
    }

    pub fn load_slot(&self, slot_id: usize) -> Result<GameState, Box<dyn std::error::Error>> {
        let conn = self.conn.lock().unwrap();
        let json: String = conn.query_row(
            "SELECT game_state_json FROM save_slots WHERE slot_id = ?1",
            params![slot_id as i64],
            |row| row.get(0),
        )?;
        let gs: GameState = serde_json::from_str(&json)?;
        Ok(gs)
    }

    pub fn get_save_slot_info(
        &self,
        slot_id: usize,
    ) -> Result<Option<SaveSlotInfo>, rusqlite::Error> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT slot_id, timestamp, score, level, lines, difficulty FROM save_slots WHERE slot_id = ?1",
        )?;
        let mut rows = stmt.query_map(params![slot_id as i64], |row| {
            Ok(SaveSlotInfo {
                slot_id: row.get::<_, i64>(0)? as usize,
                timestamp: row.get(1)?,
                score: row.get::<_, i64>(2)? as u32,
                level: row.get::<_, i64>(3)? as u32,
                lines: row.get::<_, i64>(4)? as u32,
                difficulty: row.get(5)?,
            })
        })?;

        if let Some(res) = rows.next() {
            Ok(Some(res?))
        } else {
            Ok(None)
        }
    }

    pub fn get_all_save_slots(&self) -> Vec<Option<SaveSlotInfo>> {
        let mut slots = Vec::with_capacity(5);
        for i in 1..=5 {
            slots.push(self.get_save_slot_info(i).unwrap_or(None));
        }
        slots
    }

    pub fn record_high_score(&self, gs: &GameState) -> Result<(), Box<dyn std::error::Error>> {
        let secs = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);
        let days = secs / 86400;
        let hours = (secs % 86400) / 3600;
        let mins = (secs % 3600) / 60;
        let timestamp = format!("Day {} {:02}:{:02} UTC", days, hours, mins);
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT INTO high_scores (timestamp, score, level, lines, difficulty)
             VALUES (?1, ?2, ?3, ?4, ?5)",
            params![
                timestamp,
                gs.score as i64,
                gs.level as i64,
                gs.total_lines as i64,
                gs.difficulty.as_str()
            ],
        )?;

        // Update player stats
        conn.execute(
            "UPDATE player_stats SET
                total_games_played = total_games_played + 1,
                total_lines_cleared = total_lines_cleared + ?1,
                high_score = MAX(high_score, ?2)
             WHERE id = 1",
            params![gs.total_lines as i64, gs.score as i64],
        )?;

        Ok(())
    }

    pub fn get_high_scores(&self, limit: usize) -> Vec<HighScoreEntry> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = match conn.prepare(
            "SELECT id, timestamp, score, level, lines, difficulty FROM high_scores ORDER BY score DESC, timestamp DESC LIMIT ?1"
        ) {
            Ok(stmt) => stmt,
            Err(_) => return Vec::new(),
        };

        let rows = stmt.query_map(params![limit as i64], |row| {
            Ok(HighScoreEntry {
                id: row.get(0)?,
                timestamp: row.get(1)?,
                score: row.get::<_, i64>(2)? as u32,
                level: row.get::<_, i64>(3)? as u32,
                lines: row.get::<_, i64>(4)? as u32,
                difficulty: row.get(5)?,
            })
        });

        match rows {
            Ok(mapped) => mapped.flatten().collect(),
            Err(_) => Vec::new(),
        }
    }

    pub fn get_player_stats(&self) -> PlayerStats {
        let conn = self.conn.lock().unwrap();
        conn.query_row(
            "SELECT total_games_played, total_lines_cleared, high_score FROM player_stats WHERE id = 1",
            [],
            |row| {
                Ok(PlayerStats {
                    total_games_played: row.get::<_, i64>(0)? as u32,
                    total_lines_cleared: row.get::<_, i64>(1)? as u32,
                    high_score: row.get::<_, i64>(2)? as u32,
                })
            },
        )
        .unwrap_or_default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::settings::Difficulty;

    #[test]
    fn test_db_init_and_save_load() {
        let db = Database::new_in_memory().unwrap();
        let mut gs = GameState::new(Difficulty::Moderate);
        gs.score = 12500;
        gs.level = 4;
        gs.total_lines = 32;

        db.save_slot(1, &gs).unwrap();

        let info = db.get_save_slot_info(1).unwrap().unwrap();
        assert_eq!(info.slot_id, 1);
        assert_eq!(info.score, 12500);
        assert_eq!(info.level, 4);
        assert_eq!(info.lines, 32);

        let loaded_gs = db.load_slot(1).unwrap();
        assert_eq!(loaded_gs.score, 12500);
        assert_eq!(loaded_gs.level, 4);
        assert_eq!(loaded_gs.total_lines, 32);
    }

    #[test]
    fn test_high_scores_and_stats() {
        let db = Database::new_in_memory().unwrap();
        let mut gs = GameState::new(Difficulty::Difficult);
        gs.score = 50000;
        gs.level = 10;
        gs.total_lines = 80;

        db.record_high_score(&gs).unwrap();

        let scores = db.get_high_scores(10);
        assert_eq!(scores.len(), 1);
        assert_eq!(scores[0].score, 50000);

        let stats = db.get_player_stats();
        assert_eq!(stats.total_games_played, 1);
        assert_eq!(stats.total_lines_cleared, 80);
        assert_eq!(stats.high_score, 50000);
    }
}
