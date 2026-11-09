use std::path::PathBuf;
use std::sync::Mutex;

#[derive(Default)]
pub struct WorkspaceState {
    root: Mutex<Option<PathBuf>>,
}

impl WorkspaceState {
    pub fn set(&self, root: PathBuf) -> Result<(), String> {
        let mut guard = self
            .root
            .lock()
            .map_err(|_| "État du workspace verrouillé".to_string())?;
        *guard = Some(root);
        Ok(())
    }

    pub fn root(&self) -> Result<PathBuf, String> {
        self.root
            .lock()
            .map_err(|_| "État du workspace verrouillé".to_string())?
            .clone()
            .ok_or_else(|| "Aucun dossier workspace ouvert".to_string())
    }

    pub fn try_root(&self) -> Option<PathBuf> {
        self.root.lock().ok().and_then(|guard| guard.clone())
    }
}
