use anyhow::Result;
use numpy::{PyReadonlyArray1, PyReadonlyArray2};
use pyo3::prelude::*;
//use rayon::prelude::*;
use std::{
    collections::HashMap,
    fs::File,
    io::{BufWriter, Write},
};

#[pyfunction]
fn process_hits_py(
    _py: Python,
    scores: PyReadonlyArray2<f32>,
    indices: PyReadonlyArray2<i64>, // Using i64 for indices
    query_ids_p: PyReadonlyArray1<i64>,
    target_ids_p: PyReadonlyArray1<i64>,
    output_path: String,
) -> PyResult<()> {
    let mut output = BufWriter::new(File::create(output_path)?);
    let scores_array = scores.as_array();
    let num_rows = scores_array.shape()[0] as i64;
    let num_cols = scores_array.shape()[1];
    let indices_array = indices.as_array();
    //let query_ids = query_ids_p.as_array();
    let query_ids: Vec<_> = query_ids_p.as_array().into_iter().collect();
    let target_ids: Vec<_> = target_ids_p.as_array().into_iter().collect();
    let last_target_idx = target_ids.len() - 1;
    let last_target_start = *target_ids[last_target_idx] as usize;

    for (query_idx, query_start) in query_ids.enumerate() {
        let len = &query_ids
            .get(query_idx + 1)
            .or(Some(&&num_rows))
            .map(|v| v - query_start)
            .unwrap()
            .clone();

        let mut results: HashMap<usize, f32> =
            HashMap::with_capacity(*len as usize * (num_cols as usize));

        for q_i in *query_start..(query_start + len) {
            let q_i = *q_i as usize;
            let qscores: Vec<_> = (0..num_cols)
                .into_iter()
                .map(|col| scores_array[[q_i, col]])
                .collect();

            let qindices: Vec<_> = (0..num_cols)
                .into_iter()
                .map(|col| indices_array[[q_i, col]])
                .collect();

            let targets: Vec<_> = qindices
                .iter()
                .map(|v| match target_ids.binary_search(&&v) {
                    Ok(p) => p,
                    Err(tstart) => {
                        if tstart >= last_target_start {
                            // It fell into the last target
                            last_target_idx
                        } else {
                            // Move it back one place
                            tstart - 1
                        }
                    }
                })
                .into_iter()
                .collect();

            let mut tscores: Vec<_> = targets.iter().zip(qscores).collect();
            tscores.sort_by(|a, b| {
                a.0.cmp(b.0).then_with(|| b.1.partial_cmp(&a.1).unwrap())
            });
            tscores.dedup_by(|a, b| a.0 == b.0);

            for (target_id, target_score) in tscores {
                results
                    .entry(*target_id)
                    .and_modify(|v| *v += target_score)
                    .or_insert(target_score);
            }
        }

        for (target_id, score) in &results {
            writeln!(
                output,
                "{} {} {:.7}",
                query_idx + 1,
                target_id + 1,
                score
            )?;
        }
        results.clear();
    }

    Ok(())
}

#[pymodule]
fn process_hits(_py: Python, m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(process_hits_py, m)?)?;

    Ok(())
}
