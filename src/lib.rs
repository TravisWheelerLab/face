use numpy::{PyReadonlyArray1, PyReadonlyArray2};
use pyo3::prelude::*;
use std::{
    collections::HashMap,
    fs::File,
    io::{BufWriter, Write},
    //time::Instant,
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
    // Write results to a file
    let mut output = BufWriter::new(File::create(output_path)?);

    // Access the numpy arrays safely
    let scores_array = scores.as_array();
    let num_rows = scores_array.shape()[0] as i64;
    let num_cols = scores_array.shape()[1];
    //dbg!(&scores_array);

    let indices_array = indices.as_array();
    //dbg!(&indices_array);

    let query_ids = query_ids_p.as_array();
    //dbg!(&query_ids);

    let target_ids: Vec<_> = target_ids_p.as_array().into_iter().collect();
    //dbg!(&target_ids);
    let last_target_idx = target_ids.len() - 1;
    //dbg!(&last_target_idx);
    let last_target_start = *target_ids[last_target_idx] as usize;
    //dbg!(&last_target_start);

    for (query_idx, query_start) in query_ids.iter().enumerate() {
        //if query_idx % 100 == 0 {
        //    println!("{query_idx}: {query_start}");
        //    io::stdout().flush()?;
        //}
        let len = query_ids
            .get(query_idx + 1)
            .or(Some(&num_rows))
            .map(|v| v - query_start)
            .unwrap();
        let hash_size = len as usize * (num_cols as usize);
        let mut hash: HashMap<usize, f32> = HashMap::with_capacity(hash_size);

        //println!(
        //    ">>>query_idx {query_idx} query_start {query_start} len {len:?}\n"
        //);
        for q_i in *query_start..(query_start + len) {
            let q_i = q_i as usize;
            let qscores: Vec<_> = (0..num_cols)
                .into_iter()
                .map(|col| scores_array[[q_i, col]])
                .collect();
            //println!("q_i {q_i}");
            //println!("scores: {qscores:?}");
            let qindices: Vec<_> = (0..num_cols)
                .into_iter()
                .map(|col| indices_array[[q_i, col]])
                .collect();
            //println!("indices: {qindices:?}");

            //let start = Instant::now();
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
            //times.push(start.elapsed().as_nanos());
            //println!("targets {targets:?}");

            let mut tscores: Vec<_> = targets.iter().zip(qscores).collect();
            //println!("tscores {tscores:?}");
            tscores.sort_by(|a, b| {
                a.0.cmp(b.0).then_with(|| b.1.partial_cmp(&a.1).unwrap())
            });
            //println!("sorted  {tscores:?}");
            tscores.dedup_by(|a, b| a.0 == b.0);
            //println!("dedup   {tscores:?}");
            //println!();

            for (target_id, target_score) in tscores {
                hash.entry(*target_id)
                    .and_modify(|v| *v += target_score)
                    .or_insert(target_score);
            }
        }

        for (target_id, score) in &hash {
            writeln!(
                output,
                "{} {} {:.7}",
                query_idx + 1,
                target_id + 1,
                score
            )?;
        }
        hash.clear();
        //println!();
    }
    //println!(
    //    "binary search time (seconds) {}",
    //    times.iter().sum::<u128>() as f64 / 1_000_000_000f64
    //);
    //println!();

    //let mut results: Vec<HashMap<i64, f32>> =
    //    Vec::with_capacity(query_ids.len());

    //// Initialize each hash map and add it to the vector
    //for _ in 0..query_ids.len() {
    //    let new_res: HashMap<i64, f32> = HashMap::new();
    //    results.push(new_res);
    //}

    //// Process each query
    //for (query_idx, query_id) in query_ids.iter().enumerate() {
    //    println!("{query_idx}: query_id {query_id}");
    //    for col in 0..scores_array.shape()[1] {
    //        let score = scores_array[[query_idx, col]];
    //        println!("col {col} score {score}");
    //        if score >= 0.0 {
    //            let target_idx =
    //                indices_array[[query_idx as usize, col]] as usize;

    //            let target_id = &target_ids[target_idx as usize];

    //            println!("target_idx {target_idx} target_id {target_id}\n");

    //            // Accumulate scores
    //            *results[*query_id as usize]
    //                .entry(*target_id)
    //                .or_insert(0.0) += score;
    //        }
    //    }
    //}

    //for query_id in 0..query_ids.len() {
    //    for (target_id, score) in &results[query_id] {
    //        writeln!(
    //            file,
    //            "{} {} {:.7}",
    //            query_id + 1,
    //            target_id + 1,
    //            score
    //        )?;
    //    }
    //}

    Ok(())
}

#[pymodule]
fn process_hits(_py: Python, m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(process_hits_py, m)?)?;

    Ok(())
}
