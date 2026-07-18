//! Repository Insights: statistiche calcolate localmente dal log e dall'albero
//! corrente (contributori, attività per settimana/giorno, linguaggi).

use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};

use git2::{Sort, TreeWalkMode, TreeWalkResult};

use crate::model::{Conteggio, Insights};

/// Calcola gli Insights sugli ultimi `limite` commit.
pub fn insights(percorso: &str, limite: usize) -> Result<Insights, String> {
    let repo = crate::apri(percorso)?;
    if repo.head().is_err() {
        return Ok(Insights {
            totale_commit: 0,
            contributori: Vec::new(),
            per_settimana: vec![0; 12],
            lingue: Vec::new(),
            per_giorno: vec![0; 7],
            file_caldi: Vec::new(),
            aggiunte: 0,
            rimozioni: 0,
        });
    }

    let ora = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0);

    let mut walk = repo.revwalk().map_err(|e| e.to_string())?;
    walk.push_head().map_err(|e| e.to_string())?;
    walk.set_sorting(Sort::TIME).map_err(|e| e.to_string())?;

    let mut autori: HashMap<String, usize> = HashMap::new();
    let mut per_settimana = vec![0usize; 12];
    let mut per_giorno = vec![0usize; 7]; // Lun..Dom
    let mut file_conteggi: HashMap<String, usize> = HashMap::new();
    let mut aggiunte = 0usize;
    let mut rimozioni = 0usize;
    let mut totale = 0usize;
    // L'analisi dei diff (file caldi, churn) è cara: la limitiamo ai più recenti.
    const ANALIZZA: usize = 300;

    for oid in walk.take(limite) {
        let oid = oid.map_err(|e| e.to_string())?;
        let commit = repo.find_commit(oid).map_err(|e| e.to_string())?;
        totale += 1;

        let autore = commit.author().name().unwrap_or("?").to_string();
        *autori.entry(autore).or_insert(0) += 1;

        let t = commit.time().seconds();
        // Settimane fa (0 = questa settimana), solo le ultime 12.
        let sett = ((ora - t).max(0) / (7 * 86_400)) as usize;
        if sett < 12 {
            per_settimana[11 - sett] += 1;
        }
        // Giorno della settimana con Lunedì = 0 (epoch 1970-01-01 = giovedì).
        let giorni = t.div_euclid(86_400);
        let dow = (((giorni % 7) + 3) % 7 + 7) % 7;
        per_giorno[dow as usize] += 1;

        // File toccati e churn, solo per i commit più recenti (costo contenuto).
        if totale <= ANALIZZA {
            if let Ok(albero) = commit.tree() {
                let padre = commit.parent(0).ok().and_then(|p| p.tree().ok());
                if let Ok(diff) = repo.diff_tree_to_tree(padre.as_ref(), Some(&albero), None) {
                    if let Ok(s) = diff.stats() {
                        aggiunte += s.insertions();
                        rimozioni += s.deletions();
                    }
                    for d in diff.deltas() {
                        if let Some(p) = d.new_file().path().or_else(|| d.old_file().path()) {
                            *file_conteggi.entry(p.to_string_lossy().to_string()).or_insert(0) += 1;
                        }
                    }
                }
            }
        }
    }

    let mut file_caldi: Vec<Conteggio> = file_conteggi
        .into_iter()
        .map(|(etichetta, valore)| Conteggio { etichetta, valore })
        .collect();
    file_caldi.sort_by(|a, b| b.valore.cmp(&a.valore));
    file_caldi.truncate(8);

    // Top contributori (ordinati per numero di commit, primi 8).
    let mut contributori: Vec<Conteggio> = autori
        .into_iter()
        .map(|(etichetta, valore)| Conteggio { etichetta, valore })
        .collect();
    contributori.sort_by(|a, b| b.valore.cmp(&a.valore));
    contributori.truncate(8);

    let lingue = conta_lingue(&repo).unwrap_or_default();

    Ok(Insights {
        totale_commit: totale,
        contributori,
        per_settimana,
        lingue,
        per_giorno,
        file_caldi,
        aggiunte,
        rimozioni,
    })
}

/// Conta i file dell'albero corrente per estensione (linguaggio approssimato).
fn conta_lingue(repo: &git2::Repository) -> Result<Vec<Conteggio>, String> {
    let albero = repo
        .head()
        .map_err(|e| e.to_string())?
        .peel_to_tree()
        .map_err(|e| e.to_string())?;

    let mut conteggi: HashMap<String, usize> = HashMap::new();
    albero
        .walk(TreeWalkMode::PreOrder, |_dir, entry| {
            if entry.kind() == Some(git2::ObjectType::Blob) {
                if let Some(nome) = entry.name() {
                    let ext = nome.rsplit_once('.').map(|(_, e)| e.to_lowercase());
                    let etichetta = ext.unwrap_or_else(|| "(altro)".into());
                    *conteggi.entry(etichetta).or_insert(0) += 1;
                }
            }
            TreeWalkResult::Ok
        })
        .map_err(|e| e.to_string())?;

    let mut lingue: Vec<Conteggio> = conteggi
        .into_iter()
        .map(|(etichetta, valore)| Conteggio { etichetta, valore })
        .collect();
    lingue.sort_by(|a, b| b.valore.cmp(&a.valore));
    lingue.truncate(8);
    Ok(lingue)
}
