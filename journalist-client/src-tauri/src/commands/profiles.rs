use tauri::State;

use crate::model::Profiles;

#[tauri::command]
pub async fn get_profiles(profiles: State<'_, Profiles>) -> Result<&Profiles, ()> {
    // Tauri command must return a result
    // hence the seemingly pointless wrapping in an Ok
    Ok(profiles.inner())
}
