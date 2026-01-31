/*
 * Beats TV - Premium IPTV Player
 * Secure Credential Storage Layer
 */

use keyring::Entry;
use anyhow::{Result, Context};

const SERVICE_NAME: &str = "com.beatstv.app";

/// Save a password to the system keyring
pub fn save_password(source_name: &str, password: &str) -> Result<()> {
    let entry = Entry::new(SERVICE_NAME, source_name)?;
    entry.set_password(password).context("Failed to save password to keyring")?;
    Ok(())
}

/// Retrieve a password from the system keyring
pub fn get_password(source_name: &str) -> Result<String> {
    let entry = Entry::new(SERVICE_NAME, source_name)?;
    entry.get_password().context("Failed to retrieve password from keyring")
}

/// Delete a password from the system keyring
pub fn delete_password(source_name: &str) -> Result<()> {
    let entry = Entry::new(SERVICE_NAME, source_name)?;
    let _ = entry.delete_credential(); // Ignore error if it doesn't exist
    Ok(())
}
