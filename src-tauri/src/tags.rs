use crate::types::Tag;
use anyhow::Result;
use rusqlite::Connection;
use std::collections::HashMap;

pub fn detect_tags(conn: &Connection) -> Result<Vec<Tag>> {
    let mut stmt = conn.prepare("SELECT name, hidden FROM channels")?;
    let rows = stmt.query_map([], |row| {
        let name: String = row.get(0)?;
        let hidden: bool = row.get(1)?;
        Ok((name, hidden))
    })?;

    let mut tag_counts: HashMap<String, usize> = HashMap::new();
    let mut tag_hidden_counts: HashMap<String, usize> = HashMap::new();

    let delimiters = [" | ", "|", " : ", ":", " - ", "[", "]", "(", ")"];

    for row in rows {
        if let Ok((name, hidden)) = row {
            // Heuristic: Check for common delimiters
            for delimiter in delimiters.iter() {
                if let Some(idx) = name.find(delimiter) {
                    let prefix = name[..idx].trim().to_string();
                    if !prefix.is_empty() && prefix.len() < 20 {
                        *tag_counts.entry(prefix.clone()).or_insert(0) += 1;
                        if hidden {
                            *tag_hidden_counts.entry(prefix).or_insert(0) += 1;
                        }
                        // We found a delimiter, let's stop checking others for this name purely for simplicity/primary tag
                        // Or we could check all, but usually the first one is the "Provider/Category" tag
                        break;
                    }
                }
            }
        }
    }

    let mut tags: Vec<Tag> = tag_counts
        .into_iter()
        .filter(|(_, count)| *count > 1) // Only return tags that appear more than once
        .map(|(name, count)| {
            let hidden_count = *tag_hidden_counts.get(&name).unwrap_or(&0);
            Tag {
                name,
                count,
                hidden_count,
            }
        })
        .collect();

    tags.sort_by(|a, b| b.count.cmp(&a.count)); // Sort by frequency
    Ok(tags)
}

pub fn set_tag_visibility(conn: &Connection, tag: &str, visible: bool) -> Result<usize> {
    // We update all channels that *start with* the tag + delimiter OR contain the tag in a meaningful way.
    // Implementing a safe "starts with" or "contains" logic.
    // Given the detection logic: prefix = name[..idx]
    
    // We will use a broad text match for now, improving precision if needed.
    // Matches: "TAG | ...", "TAG: ...", "[TAG] ...", "TAG - ..."
    
    // Note: This matches the heuristic from detect_tags
    
    let query = format!(
        "UPDATE channels SET hidden = ?1 WHERE \
        name LIKE ?2 OR \
        name LIKE ?3 OR \
        name LIKE ?4 OR \
        name LIKE ?5 OR \
        name LIKE ?6"
    );

    let pattern_pipe = format!("{} |%", tag);
    let pattern_colon = format!("{}:%", tag);
    let pattern_bracket = format!("[{}]%", tag);
    let pattern_dash = format!("{} -%", tag);
    let pattern_pipe_tight = format!("{}|%", tag); // Handle no spaces

    let hidden = !visible;
    let count = conn.execute(
        &query, 
        (
            hidden, 
            pattern_pipe, 
            pattern_colon, 
            pattern_bracket, 
            pattern_dash, 
            pattern_pipe_tight
        )
    )?;
    
    Ok(count)
}
