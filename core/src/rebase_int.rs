//! Rebase interattivo: riscrive una sequenza di commit applicando le mosse
//! scelte (pick / squash / reword / drop). Implementato "a mano" replicando i
//! commit sopra a una base, perché git2 non offre l'editor del piano di rebase.
//!
//! NB: pensato per storie lineari (i commit nel range non devono essere merge).
//! In caso di conflitti durante la riapplicazione, l'operazione viene annullata.

use git2::build::CheckoutBuilder;
use git2::{Commit, Oid, Repository};

use crate::model::MossaRebase;

/// Esegue il rebase interattivo. `base_id` è il commit (escluso) sotto al primo
/// da riscrivere; `mosse` sono ordinate dal più vecchio al più recente.
pub fn esegui(percorso: &str, base_id: &str, mosse: Vec<MossaRebase>) -> Result<String, String> {
    let repo = crate::apri(percorso)?;

    // Nome del ramo corrente: alla fine lo sposteremo sul nuovo ultimo commit.
    let head = repo.head().map_err(|e| e.to_string())?;
    let nome_ref = head
        .name()
        .ok_or("HEAD senza nome (sei su una testa staccata?)")?
        .to_string();

    let firma = repo
        .signature()
        .or_else(|_| git2::Signature::now("Oops", "oops@local"))
        .map_err(|e| e.to_string())?;

    let base_oid = Oid::from_str(base_id).map_err(|e| e.to_string())?;
    // `corrente` è la punta che cresce man mano che riapplichiamo i commit.
    let mut corrente = repo.find_commit(base_oid).map_err(|e| e.to_string())?;
    let mut primo_fatto = false;
    let mut applicati = 0;

    for mossa in &mosse {
        if mossa.azione == "drop" {
            continue;
        }
        let oid = Oid::from_str(&mossa.id).map_err(|e| e.to_string())?;
        let commit = repo.find_commit(oid).map_err(|e| e.to_string())?;

        // Applica le modifiche del commit sopra alla punta corrente.
        let mut indice = repo
            .cherrypick_commit(&commit, &corrente, 0, None)
            .map_err(|e| e.to_string())?;
        if indice.has_conflicts() {
            return Err(format!(
                "conflitto riapplicando {}: rebase annullato",
                &mossa.id[..7.min(mossa.id.len())]
            ));
        }
        let albero_oid = indice.write_tree_to(&repo).map_err(|e| e.to_string())?;
        let albero = repo.find_tree(albero_oid).map_err(|e| e.to_string())?;

        // "squash" sul primo commit non ha senso: lo trattiamo come "pick".
        let squash = mossa.azione == "squash" && primo_fatto;

        let nuovo = if squash {
            // Fonde nel commit precedente: stesso genitore, messaggi uniti.
            let padre_oid = corrente.parent_id(0).map_err(|e| e.to_string())?;
            let padre = repo.find_commit(padre_oid).map_err(|e| e.to_string())?;
            let messaggio = mossa.messaggio.clone().unwrap_or_else(|| {
                format!(
                    "{}\n\n{}",
                    corrente.message().unwrap_or(""),
                    commit.message().unwrap_or("")
                )
            });
            repo.commit(None, &commit.author(), &firma, &messaggio, &albero, &[&padre])
        } else {
            let messaggio = if mossa.azione == "reword" {
                mossa.messaggio.clone().unwrap_or_else(|| commit.message().unwrap_or("").to_string())
            } else {
                commit.message().unwrap_or("").to_string()
            };
            repo.commit(None, &commit.author(), &firma, &messaggio, &albero, &[&corrente])
        }
        .map_err(|e| e.to_string())?;

        corrente = repo.find_commit(nuovo).map_err(|e| e.to_string())?;
        primo_fatto = true;
        applicati += 1;
    }

    // Sposta il ramo corrente sul nuovo ultimo commit e aggiorna la cartella.
    sposta_ramo(&repo, &nome_ref, &corrente)?;
    Ok(format!("rebase interattivo completato ({applicati} commit)"))
}

/// Aggiorna il riferimento del ramo e ricarica la cartella di lavoro.
fn sposta_ramo(repo: &Repository, nome_ref: &str, punta: &Commit) -> Result<(), String> {
    repo.reference(nome_ref, punta.id(), true, "rebase interattivo")
        .map_err(|e| e.to_string())?;
    repo.set_head(nome_ref).map_err(|e| e.to_string())?;
    repo.checkout_head(Some(CheckoutBuilder::new().force()))
        .map_err(|e| e.to_string())
}

#[cfg(test)]
mod test {
    use super::*;
    use std::fs;

    #[test]
    fn squash_di_due() {
        let dir = tempfile::tempdir().unwrap();
        let p = dir.path().to_str().unwrap();
        git2::Repository::init(dir.path()).unwrap();

        // base
        fs::write(dir.path().join("a.txt"), "1\n").unwrap();
        crate::stage::aggiungi(p, "a.txt").unwrap();
        let base = crate::commit::crea(p, "base", "T", "t@t.it").unwrap();

        // c1
        fs::write(dir.path().join("b.txt"), "b\n").unwrap();
        crate::stage::aggiungi(p, "b.txt").unwrap();
        let c1 = crate::commit::crea(p, "c1", "T", "t@t.it").unwrap();

        // c2
        fs::write(dir.path().join("c.txt"), "c\n").unwrap();
        crate::stage::aggiungi(p, "c.txt").unwrap();
        let c2 = crate::commit::crea(p, "c2", "T", "t@t.it").unwrap();

        // Rebase: pick c1, squash c2 dentro c1 -> un solo commit sopra base.
        esegui(
            p,
            &base,
            vec![
                MossaRebase { id: c1, azione: "pick".into(), messaggio: None },
                MossaRebase { id: c2, azione: "squash".into(), messaggio: Some("uniti".into()) },
            ],
        )
        .unwrap();

        let titoli: Vec<String> = crate::commit::log(p, 10).unwrap().into_iter().map(|v| v.titolo).collect();
        assert_eq!(titoli, vec!["uniti", "base"]);
        // I file di entrambi i commit ci sono ancora.
        assert!(dir.path().join("b.txt").exists());
        assert!(dir.path().join("c.txt").exists());
    }
}
