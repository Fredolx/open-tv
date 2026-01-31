/*
 * Beats TV - Premium IPTV Player
 * Copyright (C) 2026 Beats TV Team
 *
 * This program is free software; you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation; either version 2 of the License, or
 * (at your option) any later version.
 *
 * This program is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 * GNU General Public License for more details.
 *
 * You should have received a copy of the GNU General Public License along
 * with this program; if not, write to the Free Software Foundation, Inc.,
 * 51 Franklin Street, Fifth Floor, Boston, MA 02110-1301 USA.
 *
 * This project is a fork of Open TV by Fredolx.
 */

#[cfg(test)]
mod tests {
    use crate::tags::{detect_tags, set_bulk_tag_visibility};
    use rusqlite::Connection;

    fn setup_db() -> Connection {
        let conn = Connection::open_in_memory().unwrap();
        conn.execute(
            "CREATE TABLE channels (
                id INTEGER PRIMARY KEY,
                name TEXT NOT NULL,
                url TEXT,
                group_title TEXT,
                media_type INTEGER DEFAULT 0,
                hidden BOOLEAN DEFAULT 0
            )",
            [],
        )
        .unwrap();
        conn
    }

    #[test]
    fn test_bulk_tag_hiding() {
        let conn = setup_db();

        // 1. Insert Sample Data
        // "Sports" tag candidates
        conn.execute("INSERT INTO channels (name, media_type) VALUES ('Sports | Channel 1', 0)", []).unwrap();
        conn.execute("INSERT INTO channels (name, media_type) VALUES ('Sports: Channel 2', 0)", []).unwrap();
        conn.execute("INSERT INTO channels (name, media_type) VALUES ('Sports - Channel 3', 0)", []).unwrap();
        // "Movies" tag candidates
        conn.execute("INSERT INTO channels (name, media_type) VALUES ('Movies | Action 1', 1)", []).unwrap();
        conn.execute("INSERT INTO channels (name, media_type) VALUES ('Movies - Comedy 1', 1)", []).unwrap();
        
        // 2. Detect Tags (Should find "Sports" and "Movies")
        let tags = detect_tags(&conn).unwrap();
        println!("Detected Tags: {:?}", tags);
        
        let sports_tag = tags.iter().find(|t| t.name == "Sports").expect("Sports tag not found");
        assert_eq!(sports_tag.count, 3, "Should have 3 Sports channels");
        assert_eq!(sports_tag.hidden_count, 0, "Initially 0 hidden");

        // 3. HIDE "Sports" (Simulation of Deselecting "Sports")
        let tags_to_hide = vec!["Sports".to_string()];
        let updated = set_bulk_tag_visibility(&conn, &tags_to_hide, false).unwrap(); // visible=false matches UI "unchecked"
        println!("Channels updated (hidden): {}", updated);
        
        // Verify they are actually hidden in DB
        let sports_hidden: i64 = conn.query_row(
            "SELECT COUNT(*) FROM channels WHERE name LIKE 'Sports%' AND hidden = 1", 
            [], 
            |row| row.get(0)
        ).unwrap();
        assert!(sports_hidden > 0, "Should have hidden sports channels");

        // 4. Verify detect_tags reflects the change
        let new_tags = detect_tags(&conn).unwrap();
        let new_sports = new_tags.iter().find(|t| t.name == "Sports").unwrap();
        println!("Sports Tag State: {:?}", new_sports);
        assert_eq!(new_sports.hidden_count, new_sports.count, "All Sports channels should be hidden");

        // 5. SHOW "Sports" again (Select All)
        set_bulk_tag_visibility(&conn, &tags_to_hide, true).unwrap();
        
        let final_tags = detect_tags(&conn).unwrap();
        let final_sports = final_tags.iter().find(|t| t.name == "Sports").unwrap();
        assert_eq!(final_sports.hidden_count, 0, "All Sports channels should be visible again");
        
        println!("Test Passed: Bulk Select/Deselect logic is correctly updating DB");
    }
}
