//! Tipi dati condivisi tra backend e frontend (serializzati in JSON).

use serde::{Deserialize, Serialize};

/// Un repository aperto di recente (mostrato nella schermata di avvio).
#[derive(Clone, Serialize, Deserialize)]
pub struct RepoRecente {
    /// Percorso assoluto della cartella del repository.
    pub percorso: String,
    /// Nome breve (l'ultima parte del percorso), comodo da mostrare.
    pub nome: String,
}

/// In quale stato si trova un file rispetto a Git.
#[derive(Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum StatoFile {
    /// Nuovo file mai tracciato.
    Nuovo,
    /// File modificato.
    Modificato,
    /// File cancellato.
    Cancellato,
    /// File rinominato.
    Rinominato,
    /// Cambio di tipo (es. da file a link).
    TipoCambiato,
    /// In conflitto dopo un merge.
    Conflitto,
}

/// Un file che compare nello stato del repository (git status).
#[derive(Clone, Serialize, Deserialize)]
pub struct FileModificato {
    /// Percorso relativo alla radice del repository.
    pub percorso: String,
    /// Come è cambiato il file.
    pub stato: StatoFile,
    /// true se la modifica è già in staging (pronta per il commit).
    pub in_stage: bool,
}

/// Stato complessivo del repository: ramo corrente e file modificati.
#[derive(Clone, Serialize, Deserialize)]
pub struct StatoRepo {
    /// Nome del ramo corrente (es. "main"), oppure "(testa staccata)".
    pub ramo: String,
    /// File con modifiche già in staging.
    pub in_stage: Vec<FileModificato>,
    /// File con modifiche non ancora in staging.
    pub non_in_stage: Vec<FileModificato>,
    /// Quanti commit siamo avanti rispetto al remoto (da spingere).
    pub avanti: usize,
    /// Quanti commit siamo indietro rispetto al remoto (da scaricare).
    pub indietro: usize,
    /// true se il repository non ha ancora nessun commit.
    pub vuoto: bool,
}

/// Una voce della cronologia (un commit).
#[derive(Clone, Serialize, Deserialize)]
pub struct VoceLog {
    /// Hash completo del commit.
    pub id: String,
    /// Hash abbreviato (primi 7 caratteri), comodo da mostrare.
    pub id_breve: String,
    /// Prima riga del messaggio (il "titolo" del commit).
    pub titolo: String,
    /// Nome dell'autore.
    pub autore: String,
    /// Email dell'autore.
    pub email: String,
    /// Data del commit come testo (AAAA-MM-GG HH:MM).
    pub data: String,
    /// Hash abbreviati dei genitori (più di uno = commit di merge).
    pub genitori: Vec<String>,
    /// Nomi di rami/tag che puntano a questo commit (decorazioni del grafo).
    pub riferimenti: Vec<String>,
}

/// Un ramo (locale o remoto).
#[derive(Clone, Serialize, Deserialize)]
pub struct Ramo {
    /// Nome del ramo (es. "main" o "origin/main").
    pub nome: String,
    /// true se è il ramo attualmente in uso.
    pub corrente: bool,
    /// true se è un ramo remoto.
    pub remoto: bool,
}

/// Una voce della lista degli stash (modifiche messe da parte).
#[derive(Clone, Serialize, Deserialize)]
pub struct VoceStash {
    /// Posizione nella pila degli stash (0 = il più recente).
    pub indice: usize,
    /// Messaggio descrittivo dello stash.
    pub messaggio: String,
}

/// Una tag (etichetta) del repository.
#[derive(Clone, Serialize, Deserialize)]
pub struct Tag {
    /// Nome della tag (es. "v1.0").
    pub nome: String,
    /// Messaggio (vuoto se è una tag leggera).
    pub messaggio: String,
    /// Hash abbreviato del commit puntato.
    pub id_breve: String,
}

/// Nome ed email dell'autore, letti/scritti dalla config di Git.
#[derive(Clone, Serialize, Deserialize)]
pub struct ConfigUtente {
    pub nome: String,
    pub email: String,
}

/// Credenziali fornite dall'utente per un'operazione di rete (tutte opzionali).
/// Non vengono mai salvate su disco: arrivano dalla UI solo per la singola
/// operazione (push/pull/clone) e poi spariscono.
#[derive(Clone, Default, Serialize, Deserialize)]
pub struct Credenziali {
    /// Nome utente (per HTTPS).
    pub utente: Option<String>,
    /// Password o token di accesso (per HTTPS).
    pub password: Option<String>,
    /// Percorso a una chiave privata SSH da usare.
    pub chiave: Option<String>,
    /// Passphrase della chiave SSH.
    pub passphrase: Option<String>,
}

/// Un remoto configurato (nome + URL).
#[derive(Clone, Serialize, Deserialize)]
pub struct Remoto {
    pub nome: String,
    pub url: String,
}

/// Una voce del reflog (la cronologia dei movimenti di HEAD).
#[derive(Clone, Serialize, Deserialize)]
pub struct VoceReflog {
    pub id_breve: String,
    pub messaggio: String,
}

/// Un sottomodulo (submodule) del repository.
#[derive(Clone, Serialize, Deserialize)]
pub struct Submodulo {
    pub nome: String,
    pub percorso: String,
    pub url: String,
}

/// Un albero di lavoro (worktree) collegato al repository.
#[derive(Clone, Serialize, Deserialize)]
pub struct VoceWorktree {
    pub nome: String,
    pub percorso: String,
}

/// Le tre versioni di un file in conflitto (per l'editor di merge a 3 vie).
#[derive(Clone, Serialize, Deserialize)]
pub struct ConflittoVersioni {
    /// Antenato comune (la base). Può mancare.
    pub base: String,
    /// La nostra versione.
    pub nostra: String,
    /// La versione in arrivo.
    pub loro: String,
    /// Il contenuto attuale nella cartella (con i marcatori <<<<<<< di Git).
    pub corrente: String,
}

/// Una mossa del rebase interattivo: cosa fare di un commit.
#[derive(Clone, Serialize, Deserialize)]
pub struct MossaRebase {
    /// Hash del commit.
    pub id: String,
    /// Azione: "pick", "squash", "reword" o "drop".
    pub azione: String,
    /// Nuovo messaggio (per reword/squash).
    pub messaggio: Option<String>,
}

/// Una riga del blame: chi e quando ha toccato l'ultima volta quella riga.
#[derive(Clone, Serialize, Deserialize)]
pub struct VoceBlame {
    /// Numero di riga (da 1).
    pub numero: usize,
    /// Hash abbreviato del commit responsabile.
    pub id_breve: String,
    /// Autore del commit.
    pub autore: String,
    /// Contenuto della riga.
    pub testo: String,
}
