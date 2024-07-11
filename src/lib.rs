//use crate::hash::Hash;
//use anyhow::bail;
use numpy::{PyReadonlyArray1, PyReadonlyArray2};
use pyo3::prelude::*;
use std::{
    collections::HashMap,
    fs::File,
    io::{BufWriter, Write},
    //time::Instant,
};

//mod hash;

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
    //let mut output2 = BufWriter::new(File::create("hasher.out")?);
    let scores_array = scores.as_array();
    let num_rows = scores_array.shape()[0] as i64;
    let num_cols = scores_array.shape()[1];
    let indices_array = indices.as_array();
    let query_ids = query_ids_p.as_array();
    let target_ids: Vec<usize> = target_ids_p
        .as_array()
        .into_iter()
        .flat_map(|v| usize::try_from(*v))
        .into_iter()
        .collect();
    let last_target_idx = target_ids.len() - 1;
    let last_target_start = target_ids[last_target_idx];

    for (query_idx, query_start) in query_ids.iter().enumerate() {
        let len = query_ids
            .get(query_idx + 1)
            .or(Some(&num_rows))
            .map(|v| v - query_start)
            .unwrap();

        let mut results: HashMap<usize, f32> =
            HashMap::with_capacity(len as usize * (num_cols as usize));

        //let mut target_hash = Hash::new(target_ids.len() * 4);
        //dbg!(&target_ids.len());
        //dbg!(&num_cols);
        //let mut target_hash = Hash::new(len as usize * num_cols * 4);

        for q_i in *query_start..(query_start + len) {
            let q_i = q_i as usize;
            let qscores: Vec<_> = (0..num_cols)
                .into_iter()
                .map(|col| scores_array[[q_i, col]])
                .collect();

            let qindices: Vec<_> = (0..num_cols)
                .into_iter()
                .flat_map(|col| usize::try_from(indices_array[[q_i, col]]))
                .collect();

            //let t = Instant::now();
            let targets: Vec<_> = qindices
                .iter()
                .map(|v| match target_ids.binary_search(v) {
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
            //let t1 = tscores.len();
            tscores.sort_by(|a, b| {
                a.0.cmp(b.0).then_with(|| b.1.partial_cmp(&a.1).unwrap())
            });
            tscores.dedup_by(|a, b| a.0 == b.0);
            //println!("{} {t1} {}", t.elapsed().as_nanos(), tscores.len());

            for (target_id, target_score) in tscores {
                //if let Ok(t) = target_hash.add(target_id + 1) {
                //    target_hash.value[t] += target_score;
                //}

                results
                    .entry(*target_id)
                    .and_modify(|v| *v += target_score)
                    .or_insert(target_score);
            }
        }

        //for (target_id, score) in &results {
        //    writeln!(
        //        output,
        //        "{} {} {:.7}",
        //        query_idx + 1,
        //        target_id + 1,
        //        score
        //    )?;
        //}

        //for (i, score) in target_hash.value.iter().enumerate() {
        //    if score > &0. {
        //        writeln!(
        //            output2,
        //            "{} {} {:.7}",
        //            query_idx + 1,
        //            &target_hash.key[i], // target ID was already incremented
        //            score
        //        )?;
        //    }
        //}
    }

    Ok(())
}

#[pymodule]
fn process_hits(_py: Python, m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(process_hits_py, m)?)?;

    Ok(())
}
