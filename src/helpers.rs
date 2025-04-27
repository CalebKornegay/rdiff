use sha2::{Sha256, Digest, digest::Output};
use std::error::Error;
use std::fs::File;
use std::io::{Read, Seek};
use diffy;

pub fn compare_hashes(v_fps: &mut Vec<File>) -> Result<(), Box<dyn Error>> {
    let mut equal = true;
    let mut hashes: Vec<Output<Sha256>> = Vec::new();
    v_fps.iter_mut()
        .for_each(|fp| {
            let mut hash = Sha256::new();
            let mut buffer = [0; 1024];
            
            loop {
                let bytes_read = fp.read(&mut buffer).unwrap();
                if bytes_read == 0 {
                    break;
                }
                hash.update(&buffer[..bytes_read]);
            }

            hashes.push(hash.finalize());
            fp.rewind().unwrap();
        });

    for i in 1..hashes.len() {
        if hashes[i] != hashes[i - 1] {
            equal = false;
        }
    }
    drop(hashes);

    if equal {
        return Err(Box::new(std::io::Error::new(std::io::ErrorKind::InvalidData, "There is no diff between the files")));
    }
    
    Ok(())
}
