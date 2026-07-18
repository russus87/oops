//! Quadro di sintesi del repository per la Dashboard ("Repository Health").
//! Una sola funzione raccoglie, riusando gli altri moduli, tutto ciò che si vuole
//! vedere appena si apre un repository: ramo, sincronia col remoto, ultimo fetch,
//! conteggi (rami/tag/stash/conflitti), stato della working tree e ultimo commit.

use std::time::UNIX_EPOCH;

use git2::BranchType;

use crate::model::{Panoramica, StatoFile};

/// Assembla la panoramica del repository.
pub fn panoramica(percorso: &str) -> Result<Panoramica, String> {
    let st = crate::repo::stato(percorso)?;
    let rami = crate::rami::lista(percorso).unwrap_or_default();
    let tag = crate::tag::lista(percorso).unwrap_or_default();
    let stash = crate::stash::lista(percorso).unwrap_or_default();
    let conflitti = crate::conflitti::lista(percorso).unwrap_or_default();
    let remoti = crate::remote::lista_dettagli(percorso).unwrap_or_default();
    let ultimo_commit = crate::commit::log(percorso, 1)
        .ok()
        .and_then(|v| v.into_iter().next());

    let rami_locali = rami.iter().filter(|r| !r.remoto).count();
    let rami_remoti = rami.iter().filter(|r| r.remoto).count();

    // Separa i non tracciati dagli altri file fuori stage (modificati/cancellati).
    let non_tracciati = st
        .non_in_stage
        .iter()
        .filter(|f| matches!(f.stato, StatoFile::Nuovo))
        .count();
    let modificati = st.non_in_stage.len() - non_tracciati;

    Ok(Panoramica {
        ramo: st.ramo,
        upstream: nome_upstream(percorso),
        avanti: st.avanti,
        indietro: st.indietro,
        vuoto: st.vuoto,
        ultimo_fetch: ultimo_fetch(percorso),
        rami_locali,
        rami_remoti,
        tag: tag.len(),
        stash: stash.len(),
        conflitti: conflitti.len(),
        in_stage: st.in_stage.len(),
        modificati,
        non_tracciati,
        remoti,
        ultimo_commit,
    })
}

/// Nome dell'upstream del ramo corrente (es. "origin/main"), se configurato.
fn nome_upstream(percorso: &str) -> Option<String> {
    let repo = crate::apri(percorso).ok()?;
    let head = repo.head().ok()?;
    let nome = head.shorthand()?;
    let locale = repo.find_branch(nome, BranchType::Local).ok()?;
    let upstream = locale.upstream().ok()?;
    upstream.name().ok().flatten().map(|s| s.to_string())
}

/// Ultimo fetch = data di modifica del file FETCH_HEAD nella cartella .git.
/// Restituisce il timestamp Unix in secondi, oppure 0 se il file non esiste
/// (non è mai stato fatto un fetch) o non è leggibile.
fn ultimo_fetch(percorso: &str) -> i64 {
    let repo = match crate::apri(percorso) {
        Ok(r) => r,
        Err(_) => return 0,
    };
    let fetch_head = repo.path().join("FETCH_HEAD");
    std::fs::metadata(&fetch_head)
        .and_then(|m| m.modified())
        .ok()
        .and_then(|t| t.duration_since(UNIX_EPOCH).ok())
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0)
}
