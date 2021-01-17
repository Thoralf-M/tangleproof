use crate::error::Result;
use crate::proof::{InclusionProof, InclusionProofJson};
use std::{
    fs::File,
    io::{prelude::*, BufWriter},
    path::Path,
};

/// Function to read a proof from a file
pub fn read_from_file<P: AsRef<Path>>(path: P) -> Result<Option<InclusionProof>> {
    let mut file = match File::open(&path) {
        Ok(file) => file,
        _ => return Ok(None),
    };
    let mut data = String::new();
    file.read_to_string(&mut data)?;
    let proof_json: InclusionProofJson = serde_json::from_str(&data)?;
    let proof = InclusionProof::from_json(proof_json)?;
    Ok(Some(proof))
}

/// Function to write a proof to a file
pub fn write_to_file<P: AsRef<Path>>(path: P, proof: InclusionProof) -> Result<()> {
    let proof_json = proof.to_json();
    let jsonvalue = serde_json::to_value(&proof_json)?;
    let file = File::create(path)?;
    let bw = BufWriter::new(file);
    serde_json::to_writer_pretty(bw, &jsonvalue)?;
    Ok(())
}
