//! Remoti: elenco, gestione e operazioni di rete (fetch, pull, push).
//!
//! Le credenziali, se non passate esplicitamente dalla UI, vengono prese
//! dall'agent SSH o dal credential helper di sistema (lo stesso che usa `git`
//! da terminale). Le credenziali passate dall'utente NON vengono mai salvate.

use std::path::Path;

use git2::build::CheckoutBuilder;
use git2::{
    Cred, CredentialType, FetchOptions, PushOptions, RemoteCallbacks, Repository,
};

use crate::model::{Credenziali, Remoto};

/// Crea i callback di rete con la gestione delle credenziali.
/// Se `cred` è presente usa quelle; altrimenti ricade su agent/credential helper.
pub fn costruisci_callbacks(cred: Option<Credenziali>) -> RemoteCallbacks<'static> {
    let mut cb = RemoteCallbacks::new();
    cb.credentials(move |url, utente, permessi| {
        if permessi.contains(CredentialType::SSH_KEY) {
            // Chiave SSH: se l'utente ha indicato un file di chiave lo usiamo,
            // altrimenti proviamo l'agent SSH.
            if let Some(c) = &cred {
                if let Some(chiave) = &c.chiave {
                    return Cred::ssh_key(
                        utente.unwrap_or("git"),
                        None,
                        Path::new(chiave),
                        c.passphrase.as_deref(),
                    );
                }
            }
            Cred::ssh_key_from_agent(utente.unwrap_or("git"))
        } else if permessi.contains(CredentialType::USER_PASS_PLAINTEXT) {
            // HTTPS: usa utente/password forniti, altrimenti il credential helper.
            if let Some(c) = &cred {
                if let (Some(u), Some(p)) = (&c.utente, &c.password) {
                    return Cred::userpass_plaintext(u, p);
                }
            }
            let cfg = git2::Config::open_default()?;
            Cred::credential_helper(&cfg, url, utente)
        } else {
            Cred::default()
        }
    });
    cb
}

/// Elenca i nomi dei remoti configurati (di solito "origin").
pub fn lista(percorso: &str) -> Result<Vec<String>, String> {
    let repo = crate::apri(percorso)?;
    let remoti = repo.remotes().map_err(|e| e.to_string())?;
    Ok(remoti.iter().flatten().map(|s| s.to_string()).collect())
}

/// Elenca i remoti con nome e URL.
pub fn lista_dettagli(percorso: &str) -> Result<Vec<Remoto>, String> {
    let repo = crate::apri(percorso)?;
    let nomi = repo.remotes().map_err(|e| e.to_string())?;
    let mut out = Vec::new();
    for nome in nomi.iter().flatten() {
        let url = repo
            .find_remote(nome)
            .ok()
            .and_then(|r| r.url().map(|s| s.to_string()))
            .unwrap_or_default();
        out.push(Remoto {
            nome: nome.to_string(),
            url,
        });
    }
    Ok(out)
}

/// Aggiunge un nuovo remoto.
pub fn aggiungi(percorso: &str, nome: &str, url: &str) -> Result<(), String> {
    let repo = crate::apri(percorso)?;
    repo.remote(nome, url).map(|_| ()).map_err(|e| e.to_string())
}

/// Cambia l'URL di un remoto esistente.
pub fn imposta_url(percorso: &str, nome: &str, url: &str) -> Result<(), String> {
    let repo = crate::apri(percorso)?;
    repo.remote_set_url(nome, url).map_err(|e| e.to_string())
}

/// Rimuove un remoto.
pub fn rimuovi(percorso: &str, nome: &str) -> Result<(), String> {
    let repo = crate::apri(percorso)?;
    repo.remote_delete(nome).map_err(|e| e.to_string())
}

/// Carica tutte le tag sul remoto.
pub fn push_tags(percorso: &str, remoto: &str, cred: Option<Credenziali>) -> Result<(), String> {
    let repo = crate::apri(percorso)?;
    let mut r = repo.find_remote(remoto).map_err(|e| e.to_string())?;
    let mut po = PushOptions::new();
    po.remote_callbacks(costruisci_callbacks(cred));
    r.push(&["refs/tags/*:refs/tags/*"], Some(&mut po))
        .map_err(|e| e.to_string())
}

/// Elimina un ramo sul remoto (git push origin --delete <ramo>).
pub fn elimina_ramo_remoto(
    percorso: &str,
    remoto: &str,
    ramo: &str,
    cred: Option<Credenziali>,
) -> Result<(), String> {
    let repo = crate::apri(percorso)?;
    let mut r = repo.find_remote(remoto).map_err(|e| e.to_string())?;
    let mut po = PushOptions::new();
    po.remote_callbacks(costruisci_callbacks(cred));
    // Una refspec con il lato sinistro vuoto cancella il ref remoto.
    r.push(&[format!(":refs/heads/{ramo}")], Some(&mut po))
        .map_err(|e| e.to_string())
}

/// Scarica gli aggiornamenti dal remoto senza toccare i file (git fetch).
pub fn fetch(percorso: &str, remoto: &str, cred: Option<Credenziali>) -> Result<(), String> {
    let repo = crate::apri(percorso)?;
    let mut r = repo.find_remote(remoto).map_err(|e| e.to_string())?;
    let mut fo = FetchOptions::new();
    fo.remote_callbacks(costruisci_callbacks(cred));
    // Passando una lista vuota di refspec, git2 usa quelle del remoto.
    let refspec: [&str; 0] = [];
    r.fetch(&refspec, Some(&mut fo), None).map_err(|e| e.to_string())
}

/// Scarica e integra le modifiche del ramo corrente (git pull).
/// `strategia`: "ff" (solo fast-forward), "merge" (crea un commit di merge sui
/// rami divergenti) o "rebase" (riapplica i tuoi commit sopra al remoto).
pub fn pull(
    percorso: &str,
    remoto: &str,
    strategia: &str,
    cred: Option<Credenziali>,
) -> Result<String, String> {
    let repo = crate::apri(percorso)?;

    // Prima un fetch.
    fetch(percorso, remoto, cred)?;

    // Ramo corrente e suo corrispondente remoto.
    let head = repo.head().map_err(|e| e.to_string())?;
    let ramo = head.shorthand().ok_or("ramo corrente sconosciuto")?.to_string();

    let rif_remoto = format!("refs/remotes/{remoto}/{ramo}");
    let riferimento = repo
        .find_reference(&rif_remoto)
        .map_err(|_| format!("nessun ramo remoto {remoto}/{ramo}"))?;
    let annotato = repo
        .reference_to_annotated_commit(&riferimento)
        .map_err(|e| e.to_string())?;

    let (analisi, _) = repo.merge_analysis(&[&annotato]).map_err(|e| e.to_string())?;

    if analisi.is_up_to_date() {
        return Ok("già aggiornato".into());
    }
    if analisi.is_fast_forward() {
        let nome_head = head.name().ok_or("HEAD senza nome")?.to_string();
        let mut rif = repo.find_reference(&nome_head).map_err(|e| e.to_string())?;
        rif.set_target(annotato.id(), "pull fast-forward")
            .map_err(|e| e.to_string())?;
        repo.set_head(&nome_head).map_err(|e| e.to_string())?;
        repo.checkout_head(Some(CheckoutBuilder::new().force()))
            .map_err(|e| e.to_string())?;
        return Ok("aggiornato (fast-forward)".into());
    }

    // I due rami divergono: si decide in base alla strategia scelta.
    match strategia {
        "merge" => pull_merge(&repo, &annotato, remoto, &ramo),
        "rebase" => pull_rebase(&repo, &annotato),
        _ => Err("i rami divergono: scegli pull con 'merge' o 'rebase'".into()),
    }
}

/// Pull divergente con merge: crea un commit di merge col remoto.
fn pull_merge(
    repo: &Repository,
    annotato: &git2::AnnotatedCommit,
    remoto: &str,
    ramo: &str,
) -> Result<String, String> {
    repo.merge(&[annotato], None, None).map_err(|e| e.to_string())?;
    if repo.index().map_err(|e| e.to_string())?.has_conflicts() {
        return Err("merge con conflitti: risolvi i file e fai un commit".into());
    }
    let albero_id = repo
        .index()
        .map_err(|e| e.to_string())?
        .write_tree()
        .map_err(|e| e.to_string())?;
    let albero = repo.find_tree(albero_id).map_err(|e| e.to_string())?;
    let firma = repo
        .signature()
        .or_else(|_| git2::Signature::now("Oops", "oops@local"))
        .map_err(|e| e.to_string())?;
    let nostro = repo.head().map_err(|e| e.to_string())?.peel_to_commit().map_err(|e| e.to_string())?;
    let loro = repo.find_commit(annotato.id()).map_err(|e| e.to_string())?;
    repo.commit(
        Some("HEAD"),
        &firma,
        &firma,
        &format!("Merge di {remoto}/{ramo}"),
        &albero,
        &[&nostro, &loro],
    )
    .map_err(|e| e.to_string())?;
    repo.cleanup_state().map_err(|e| e.to_string())?;
    Ok("aggiornato (merge)".into())
}

/// Pull divergente con rebase: riapplica i commit locali sopra al remoto.
fn pull_rebase(repo: &Repository, annotato: &git2::AnnotatedCommit) -> Result<String, String> {
    let firma = repo
        .signature()
        .or_else(|_| git2::Signature::now("Oops", "oops@local"))
        .map_err(|e| e.to_string())?;
    let mut rb = repo
        .rebase(None, Some(annotato), None, None)
        .map_err(|e| e.to_string())?;
    loop {
        match rb.next() {
            None => break,
            Some(Ok(_op)) => {}
            Some(Err(e)) => {
                let _ = rb.abort();
                return Err(e.to_string());
            }
        }
        if repo.index().map_err(|e| e.to_string())?.has_conflicts() {
            let _ = rb.abort();
            return Err("rebase con conflitti: annullato".into());
        }
        rb.commit(None, &firma, None).map_err(|e| e.to_string())?;
    }
    rb.finish(Some(&firma)).map_err(|e| e.to_string())?;
    Ok("aggiornato (rebase)".into())
}

/// Carica i commit del ramo corrente sul remoto (git push). Con `forza=true`
/// riscrive la storia sul remoto (usare con cautela).
pub fn push(
    percorso: &str,
    remoto: &str,
    forza: bool,
    cred: Option<Credenziali>,
) -> Result<(), String> {
    let repo = crate::apri(percorso)?;
    let head = repo.head().map_err(|e| e.to_string())?;
    let ramo = head.shorthand().ok_or("ramo corrente sconosciuto")?;

    let mut r = repo.find_remote(remoto).map_err(|e| e.to_string())?;
    let mut po = PushOptions::new();
    po.remote_callbacks(costruisci_callbacks(cred));

    // Il "+" iniziale indica un push forzato.
    let prefisso = if forza { "+" } else { "" };
    let refspec = format!("{prefisso}refs/heads/{ramo}:refs/heads/{ramo}");
    r.push(&[&refspec], Some(&mut po)).map_err(|e| e.to_string())?;

    // Collega il ramo locale al suo upstream, così conosciamo l'avanti/indietro.
    imposta_upstream(&repo, ramo, remoto);
    Ok(())
}

/// Imposta l'upstream del ramo locale (branch.<ramo>.remote/merge in config).
fn imposta_upstream(repo: &Repository, ramo: &str, remoto: &str) {
    if let Ok(mut b) = repo.find_branch(ramo, git2::BranchType::Local) {
        let _ = b.set_upstream(Some(&format!("{remoto}/{ramo}")));
    }
}
