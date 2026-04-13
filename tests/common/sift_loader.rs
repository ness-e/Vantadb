#![allow(dead_code)]

use std::fs::File;
use std::io::{Read, BufReader};
use std::path::Path;

/// Parses an `.fvecs` file into a `Vec<Vec<f32>>`.
/// Format: The file consists of sequences of (d, v_1, ..., v_d).
/// Where `d` is the dimension (i32) and `v_i` are vector elements (f32).
pub fn read_fvecs<P: AsRef<Path>>(path: P) -> std::io::Result<Vec<Vec<f32>>> {
    let mut file = BufReader::new(File::open(path)?);
    let mut results = Vec::new();
    let mut d_buf = [0u8; 4];

    loop {
        if let Err(e) = file.read_exact(&mut d_buf) {
            if e.kind() == std::io::ErrorKind::UnexpectedEof {
                // Natural end of file
                break;
            } else {
                return Err(e);
            }
        }

        let d = i32::from_le_bytes(d_buf) as usize;
        let mut vector_buf = vec![0u8; d * 4];
        file.read_exact(&mut vector_buf)?;

        let mut vector = Vec::with_capacity(d);
        for i in 0..d {
            let val_bytes = [
                vector_buf[i * 4],
                vector_buf[i * 4 + 1],
                vector_buf[i * 4 + 2],
                vector_buf[i * 4 + 3],
            ];
            vector.push(f32::from_le_bytes(val_bytes));
        }

        results.push(vector);
    }

    Ok(results)
}

/// Parses an `.ivecs` file into a `Vec<Vec<usize>>`.
/// Same structure but values are i32 instead of f32.
pub fn read_ivecs<P: AsRef<Path>>(path: P) -> std::io::Result<Vec<Vec<usize>>> {
    let mut file = BufReader::new(File::open(path)?);
    let mut results = Vec::new();
    let mut d_buf = [0u8; 4];

    loop {
        if let Err(e) = file.read_exact(&mut d_buf) {
            if e.kind() == std::io::ErrorKind::UnexpectedEof {
                break;
            } else {
                return Err(e);
            }
        }

        let d = i32::from_le_bytes(d_buf) as usize;
        let mut vector_buf = vec![0u8; d * 4];
        file.read_exact(&mut vector_buf)?;

        let mut vector = Vec::with_capacity(d);
        for i in 0..d {
            let val_bytes = [
                vector_buf[i * 4],
                vector_buf[i * 4 + 1],
                vector_buf[i * 4 + 2],
                vector_buf[i * 4 + 3],
            ];
            let val = i32::from_le_bytes(val_bytes) as usize;
            vector.push(val);
        }

        results.push(vector);
    }

    Ok(results)
}
