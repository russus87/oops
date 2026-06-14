//! Cuore di Oops: logica Git riutilizzabile, costruita su libgit2 (crate `git2`).
//! Niente dipendenze da Tauri, così resta testabile e riusabile.
//!
//! - `model`   tipi dati condivisi col frontend (serializzati in JSON)
//! - `repo`    apertura/clonazione del repository e stato dei file
//! - `stage`   mettere/togliere file dall'area di staging, scartare modifiche
//! - `commit`  cronologia (log) e creazione di commit
//! - `rami`    rami: elenco, creazione, checkout, eliminazione, merge
//! - `diff`    differenze di un file o di un commit (testo unificato)
//! - `remote`  remoti: elenco, fetch, pull, push
//! - `stash`   modifiche messe da parte (stash)
//! - `tag`     etichette (tag) del repository
//! - `azioni`  reset, cherry-pick, config dell'autore
//! - `storage` elenco dei repository aperti di recente (file JSON)

pub mod azioni;
pub mod commit;
pub mod diff;
pub mod model;
pub mod rami;
pub mod remote;
pub mod repo;
pub mod stage;
pub mod stash;
pub mod storage;
pub mod tag;

use git2::Repository;
use std::path::Path;

/// Apre il repository alla cartella indicata (cercandolo anche nei genitori).
/// Funzione comoda usata da tutti i moduli.
pub fn apri(percorso: &str) -> Result<Repository, String> {
    Repository::discover(Path::new(percorso)).map_err(|e| e.to_string())
}
