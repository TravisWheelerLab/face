use numpy::{PyReadonlyArray1, PyReadonlyArray2};
use pyo3::prelude::*;
use std::{collections::HashMap, fs::File, io::Write};

#[pyfunction]
fn process_hits_py(
    _py: Python,
    scores: PyReadonlyArray2<f32>,
    indices: PyReadonlyArray2<i64>, // Using i64 for indices
    query_ids_p: PyReadonlyArray1<i32>,
    target_ids_p: PyReadonlyArray1<i32>,
    output_path: String,
) -> PyResult<()> {
    // Access the numpy arrays safely
    let scores_array = scores.as_array();
    let indices_array = indices.as_array();

    let query_ids = query_ids_p.as_array();
    let target_ids = target_ids_p.as_array();

    let mut results: Vec<HashMap<i32, f32>> =
        Vec::with_capacity(query_ids.len());

    // Initialize each hash map and add it to the vector
    for _ in 0..query_ids.len() {
        let new_res: HashMap<i32, f32> = HashMap::new();
        results.push(new_res);
    }

    // Process each query
    for (query_idx, query_id) in query_ids.iter().enumerate() {
        for col in 0..scores_array.shape()[1] {
            let score = scores_array[[query_idx, col]];
            if score >= 0.0 {
                let target_idx =
                    indices_array[[query_idx as usize, col]] as usize;

                let target_id = &target_ids[target_idx as usize];

                // Accumulate scores
                *results[*query_id as usize]
                    .entry(*target_id)
                    .or_insert(0.0) += score;
            }
        }
    }

    // Write results to a file
    let mut file = File::create(output_path)?;

    for query_id in 0..query_ids.len() {
        for (target_id, score) in &results[query_id] {
            writeln!(
                file,
                "{} {} {:.7}",
                query_id + 1,
                target_id + 1,
                score
            )?;
        }
    }

    Ok(())
}

#[pymodule]
fn process_hits(_py: Python, m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(process_hits_py, m)?)?;

    Ok(())
}
