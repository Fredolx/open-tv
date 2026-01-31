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

use crate::types::Tag;
use anyhow::Result;
use rusqlite::{Connection, params, Transaction};
use std::collections::HashMap;

use regex::Regex;

pub fn detect_tags(conn: &Connection) -> Result<Vec<Tag>> {
    let mut stmt = conn.prepare("SELECT name, hidden, media_type FROM channels")?;
    let rows = stmt.query_map([], |row| {
        let name: String = row.get(0)?;
        let hidden: bool = row.get(1)?;
        let media_type: u8 = row.get(2).unwrap_or(0);
        Ok((name, hidden, media_type))
    })?;

    let mut tag_counts: HashMap<String, usize> = HashMap::new();
    let mut tag_hidden_counts: HashMap<String, usize> = HashMap::new();
    let mut tag_live_counts: HashMap<String, usize> = HashMap::new();
    let mut tag_vod_counts: HashMap<String, usize> = HashMap::new();
    let mut tag_series_counts: HashMap<String, usize> = HashMap::new();

    // Regex to find tags:
    // 1. Matches [TAG] or (TAG)
    // 2. Matches prefixes ending in : or | or -
    let re_brackets = Regex::new(r"\[(.*?)\]|\((.*?)\)").unwrap();
    let re_prefix = Regex::new(r"^([A-Za-z0-9 ]{2,15})(:|\|| - )").unwrap();

    // List of common ISO country codes and languages (and common IPTV variations)
    // Extended list to cover major regions.
    let valid_tags: Vec<&str> = vec![
        // Languages
        "EN", "English", "MULTI-LANG",
        // North America
        "USA", "US", "United States", "CA", "Canada", "MX", "Mexico",
        // Europe
        "UK", "Great Britain", "England", "IE", "Ireland",
        "FR", "France", "French", "DE", "Germany", "German",
        "IT", "Italy", "Italian", "ES", "Spain", "Spanish",
        "PT", "Portugal", "Portuguese", "NL", "Netherlands", "Dutch",
        "BE", "Belgium", "CH", "Switzerland", "AT", "Austria",
        "SE", "Sweden", "NO", "Norway", "DK", "Denmark", "FI", "Finland",
        "PL", "Poland", "Polish", "CZ", "Czech", "HU", "Hungary",
        "RO", "Romania", "BG", "Bulgaria", "GR", "Greece", "Greek",
        "TR", "Turkey", "Turkish", "RU", "Russia", "Russian",
        "UA", "Ukraine",
        // Asia
        "CN", "China", "Chinese", "JP", "Japan", "Japanese",
        "KR", "Korea", "Korean", "IN", "India", "Hindi",
        "TH", "Thailand", "VN", "Vietnam", "PH", "Philippines",
        "ID", "Indonesia", "MY", "Malaysia", "SG", "Singapore",
        "HK", "Hong Kong", "TW", "Taiwan",
        // South America / LATAM
        "BR", "Brazil", "AR", "Argentina", "CO", "Colombia",
        "CL", "Chile", "PE", "Peru", "UY", "Uruguay",
        "LATAM", "Latin",
        // Middle East
        "AE", "UAE", "SA", "Saudi Arabia", "Arabic", "Arab",
        "IL", "Israel", "Hebrew", "IR", "Iran", "Persian",
        // Oceania
        "AU", "Australia", "NZ", "New Zealand",
        // Africa
        "ZA", "South Africa", "Africa",
        "VIP", "PPV"
    ];

    for row in rows {
        if let Ok((name, hidden, media_type)) = row {
            let mut found_tags = Vec::new();

            // 1. Look for [Tag] or (Tag)
            for caps in re_brackets.captures_iter(&name) {
                if let Some(m) = caps.get(1).or(caps.get(2)) {
                     let tag = m.as_str().trim().to_string();
                     // Filter: Must be in our valid list (case-insensitive check)
                     if !tag.is_empty() && valid_tags.iter().any(|&vt| vt.eq_ignore_ascii_case(&tag)) {
                        found_tags.push(tag); // Keep original casing for now
                     }
                }
            }

            // 2. Look for Prefix if no bracket tags found
            if let Some(caps) = re_prefix.captures(&name) {
                if let Some(m) = caps.get(1) {
                    let tag = m.as_str().trim().to_string();
                     if !tag.is_empty() && valid_tags.iter().any(|&vt| vt.eq_ignore_ascii_case(&tag)) {
                        found_tags.push(tag);
                     }
                }
            }
            
            // Deduplicate tags for this channel (e.g. [UK] UK: Channel 1 -> just UK)
            found_tags.sort();
            found_tags.dedup();

            for tag in found_tags {
                *tag_counts.entry(tag.clone()).or_insert(0) += 1;
                 if hidden {
                    *tag_hidden_counts.entry(tag.clone()).or_insert(0) += 1;
                }
                match media_type {
                    0 => *tag_live_counts.entry(tag.clone()).or_insert(0) += 1,
                    1 => *tag_vod_counts.entry(tag.clone()).or_insert(0) += 1,
                    2 => *tag_series_counts.entry(tag).or_insert(0) += 1,
                    _ => {}
                }
            }
        }
    }

    let mut tags: Vec<Tag> = tag_counts
        .into_iter()
        .filter(|(_, count)| *count > 1) // Only return tags that appear more than once
        .map(|(name, count)| {
            let hidden_count = *tag_hidden_counts.get(&name).unwrap_or(&0);
            let count_live = *tag_live_counts.get(&name).unwrap_or(&0);
            let count_vod = *tag_vod_counts.get(&name).unwrap_or(&0);
            let count_series = *tag_series_counts.get(&name).unwrap_or(&0);
            Tag {
                name,
                count,
                hidden_count,
                count_live,
                count_vod,
                count_series,
            }
        })
        .collect();

    tags.sort_by(|a, b| {
        let priority_tags = ["USA", "US", "United States", "English", "EN"];
        let a_priority = priority_tags.iter().any(|&p| p.eq_ignore_ascii_case(&a.name));
        let b_priority = priority_tags.iter().any(|&p| p.eq_ignore_ascii_case(&b.name));

        if a_priority && !b_priority {
            std::cmp::Ordering::Less
        } else if !a_priority && b_priority {
            std::cmp::Ordering::Greater
        } else {
            b.count.cmp(&a.count)
        }
    });
    Ok(tags)
}

pub fn set_tag_visibility(conn: &Connection, tag: &str, visible: bool) -> Result<usize> {
    // We update all channels that match our detection logic for this tag.
    
    let query = "UPDATE channels SET hidden = ?1 WHERE \
        name LIKE ?2 OR \
        name LIKE ?3 OR \
        name LIKE ?4 OR \
        name LIKE ?5 OR \
        name LIKE ?6 OR \
        name LIKE ?7";

    let pattern_pipe = format!("{} |%", tag);
    let pattern_colon = format!("{}:%", tag);
    let pattern_dash = format!("{} -%", tag);
    let pattern_pipe_tight = format!("{}|%", tag); 
    
    // Tag inside brackets/parens can be anywhere, but let's assume standard position or start
    // Actually, detect_tags looks for regex \[tag\] anywhere. 
    // SQLite LIKE '%[tag]%' is safe enough for "contains [tag]"
    let pattern_bracket = format!("%[{}]%", tag);
    let pattern_paren = format!("%({})%", tag);

    let hidden = !visible;
    let count = conn.execute(
        query, 
        params![
            hidden, 
            pattern_pipe, 
            pattern_colon, 
            pattern_dash, 
            pattern_pipe_tight,
            pattern_bracket,
            pattern_paren
        ]
    )?;
    
    Ok(count)
}

pub fn set_bulk_tag_visibility(conn: &Transaction, tags: &[String], visible: bool) -> Result<usize> {
    if tags.is_empty() {
        return Ok(0);
    }
    
    let mut total_count = 0;
    
    for tag in tags {
        total_count += set_tag_visibility(conn, tag, visible)?;
    }
    
    Ok(total_count)
}
