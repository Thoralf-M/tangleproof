use crate::error::Result;
use crate::proof::InclusionProof;
use std::fs::File;
use std::io::prelude::*;
use std::io::BufWriter;
use std::path::Path;

pub fn read_from_file<P: AsRef<Path>>(path: P) -> Result<Option<InclusionProof>> {
    let mut file = match File::open(&path) {
        Ok(file) => file,
        _ => return Ok(None),
    };
    let mut data = String::new();
    file.read_to_string(&mut data)?;
    let p: InclusionProof = serde_json::from_str(&data)?;
    Ok(Some(p))
}

pub fn write_to_file<P: AsRef<Path>>(path: P, proof: InclusionProof) -> Result<()> {
    // Serialize it to a JSON string.
    let jsonvalue = serde_json::to_value(&proof)?;
    let file = File::create(path)?;
    let bw = BufWriter::new(file);
    serde_json::to_writer(bw, &jsonvalue)?;
    Ok(())
}
