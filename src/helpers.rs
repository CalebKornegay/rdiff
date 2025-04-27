use sha2::{Sha256, Digest, digest::Output};
use std::error::Error;
use std::fs::File;
use std::io::{BufReader, BufRead, Read, Seek};

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

pub fn get_max_line_count(v_fps: &Vec<File>) -> usize  {
    // Get the file size of each so we can see the max
    let lc_1: usize = BufReader::new(&v_fps[0])
    .lines().filter_map(Result::ok).count();
    let lc_2: usize = BufReader::new(&v_fps[1])
        .lines().filter_map(Result::ok).count();
    let lc_3: usize = {
        if v_fps.len() == 3 {
            BufReader::new(&v_fps[2])
            .lines().filter_map(Result::ok).count()
        } else {
            0
        }
    };

    std::cmp::max(std::cmp::max(lc_1, lc_2), lc_3)
}
