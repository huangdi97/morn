use morn_kernel::error::{Error, Result};

#[cfg(feature = "domain-biolab")]
use {
    morn_biolab_reference::BioLabDatasetProvenance,
    morn_kernel::time::Timestamp,
    serde_json::json,
    sha2::{Digest, Sha256},
    std::{fs, io::Read},
};

#[cfg(feature = "domain-biolab")]
fn sha256_file(path: &str) -> Result<String> {
    let mut file = fs::File::open(path)
        .map_err(|error| Error::external(format!("cannot open BioLab dataset {path:?}: {error}")))?;
    let mut hasher = Sha256::new();
    let mut buffer = [0_u8; 1024 * 1024];
    loop {
        let read = file.read(&mut buffer).map_err(|error| {
            Error::external(format!("cannot read BioLab dataset {path:?}: {error}"))
        })?;
        if read == 0 {
            break;
        }
        hasher.update(&buffer[..read]);
    }
    Ok(format!("sha256:{:x}", hasher.finalize()))
}

#[cfg(feature = "domain-biolab")]
fn run() -> Result<()> {
    let provenance_path = std::env::var("MORN_BIOLAB_DATASET_PROVENANCE_FILE")
        .map_err(|_| Error::validation("MORN_BIOLAB_DATASET_PROVENANCE_FILE is required"))?;
    let dataset_path = std::env::var("MORN_BIOLAB_DATASET_FILE")
        .map_err(|_| Error::validation("MORN_BIOLAB_DATASET_FILE is required"))?;
    let trusted_issuer = std::env::var("MORN_BIOLAB_DATASET_PROVENANCE_ISSUER")
        .map_err(|_| Error::validation("MORN_BIOLAB_DATASET_PROVENANCE_ISSUER is required"))?;

    if trusted_issuer.trim().is_empty() {
        return Err(Error::validation(
            "MORN_BIOLAB_DATASET_PROVENANCE_ISSUER must not be empty",
        ));
    }

    let raw = fs::read_to_string(&provenance_path).map_err(|error| {
        Error::external(format!(
            "cannot read BioLab dataset provenance file {provenance_path:?}: {error}"
        ))
    })?;
    let provenance: BioLabDatasetProvenance = serde_json::from_str(&raw).map_err(|error| {
        Error::validation(format!(
            "invalid BioLab dataset provenance JSON in {provenance_path:?}: {error}"
        ))
    })?;
    provenance.validate(Timestamp::now())?;

    if provenance.issuer != trusted_issuer.trim() {
        return Err(Error::validation(format!(
            "BioLab provenance issuer {:?} does not match deployment-trusted issuer {:?}",
            provenance.issuer,
            trusted_issuer.trim()
        )));
    }

    let actual_digest = sha256_file(&dataset_path)?;
    if actual_digest != provenance.content_digest {
        return Err(Error::validation(format!(
            "BioLab dataset digest mismatch: provenance pins {}, actual bytes are {}",
            provenance.content_digest, actual_digest
        )));
    }

    let metadata = fs::metadata(&dataset_path).map_err(|error| {
        Error::external(format!("cannot stat BioLab dataset {dataset_path:?}: {error}"))
    })?;
    if !metadata.is_file() || metadata.len() == 0 {
        return Err(Error::validation(
            "BioLab pilot dataset must be a non-empty regular file",
        ));
    }

    println!(
        "{}",
        serde_json::to_string_pretty(&json!({
            "gate": "MORN_BIOLAB_REAL_DATASET_PROVENANCE_VERIFY",
            "status": "PASS",
            "dataset_identity": provenance.canonical_identity(),
            "dataset_bytes": metadata.len(),
            "provenance": provenance,
            "claims": {
                "dataset_bytes_digest_verified": true,
                "provenance_schema_validated": true,
                "issuer_identity_pinned": true,
                "lawful_use_minted_by_morn": false,
                "scientific_outcome_proven": false,
                "customer_acceptance_proven": false
            },
            "note": "This gate binds the exact dataset bytes to deployment-owned provenance. It does not itself prove lawful use beyond the referenced governance/consent artifacts, scientific validity, an accepted outcome or customer value."
        }))
        .map_err(|error| Error::internal(error.to_string()))?
    );
    Ok(())
}

#[cfg(feature = "domain-biolab")]
fn main() -> Result<()> {
    run()
}

#[cfg(not(feature = "domain-biolab"))]
fn main() -> Result<()> {
    Err(Error::invalid_state(
        "biolab_dataset_verify requires the domain-biolab feature",
    ))
}
