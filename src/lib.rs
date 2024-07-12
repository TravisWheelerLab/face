use anyhow::Result;
use ndarray::{ArrayView, Dim};
use numpy::{PyReadonlyArray1, PyReadonlyArray2};
use pyo3::prelude::*;
use rayon::prelude::*;
use std::{
    collections::HashMap,
    fs::File,
    io::{BufWriter, Write},
    sync::{Arc, Mutex},
};

struct Args<'a> {
    query_idx: usize,
    scores: ArrayView<'a, f32, Dim<[usize; 2]>>,
    indices: ArrayView<'a, i64, Dim<[usize; 2]>>,
    query_ids: ArrayView<'a, i64, Dim<[usize; 1]>>,
    target_ids: &'a [usize],
    dedup: bool,
    output: Option<Arc<Mutex<Box<dyn Write + Send + Sync>>>>,
}

// --------------------------------------------------
#[pyfunction]
fn process_hits_py(
    _py: Python,
    scores: PyReadonlyArray2<f32>,
    indices: PyReadonlyArray2<i64>, // Using i64 for indices
    query_ids_p: PyReadonlyArray1<i64>,
    target_ids_p: PyReadonlyArray1<i64>,
    output_path: String,
    num_threads: usize,
) -> PyResult<()> {
    // Set threads
    rayon::ThreadPoolBuilder::new()
        .num_threads(num_threads)
        .build_global()
        .unwrap();

    let output: Arc<Mutex<Box<dyn Write + Send + Sync>>> = Arc::new(
        Mutex::new(Box::new(BufWriter::new(File::create(output_path)?))),
    );
    let scores_array = scores.as_array();
    //let num_rows = scores_array.shape()[0] as i64;
    //let num_cols = scores_array.shape()[1];
    let indices_array = indices.as_array();
    let query_ids = query_ids_p.as_array();
    let target_ids: Vec<usize> = target_ids_p
        .as_array()
        .into_iter()
        .flat_map(|v| usize::try_from(*v))
        .into_iter()
        .collect();
    //let last_target_idx = target_ids.len() - 1;
    //let last_target_start = target_ids[last_target_idx];

    (0..query_ids.len()).into_par_iter().for_each(|i| {
        if let Err(e) = run(Args {
            query_idx: i,
            scores: scores_array,
            indices: indices_array,
            query_ids,
            target_ids: &target_ids,
            dedup: false,
            output: Some(output.clone()),
            //output: None,
        }) {
            println!("{e}");
        }
    });

    Ok(())
}

// --------------------------------------------------
fn run(args: Args) -> Result<()> {
    let query_start = args.query_ids[args.query_idx];
    let last_target_idx = args.target_ids.len() - 1;
    let last_target_start = args.target_ids[last_target_idx];
    let num_rows = args.scores.shape()[0] as i64;
    let num_cols = args.scores.shape()[1];
    let query_len = args
        .query_ids
        .get(args.query_idx + 1)
        .or(Some(&num_rows))
        .map(|v| v - query_start)
        .unwrap();

    let mut results: HashMap<usize, f32> =
        HashMap::with_capacity(query_len as usize * (num_cols as usize));

    for q_i in query_start..(query_start + query_len) {
        let q_i = q_i as usize;
        let qscores: Vec<_> = (0..num_cols)
            .into_iter()
            .map(|col| args.scores[[q_i, col]])
            .collect();

        let qindices: Vec<_> = (0..num_cols)
            .into_iter()
            .flat_map(|col| usize::try_from(args.indices[[q_i, col]]))
            .collect();

        let targets: Vec<_> = qindices
            .iter()
            .map(|v| match args.target_ids.binary_search(v) {
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

        // Create [(target_id, score)]
        let mut tscores: Vec<_> = targets.iter().zip(qscores).collect();
        if args.dedup {
            tscores.sort_by(|a, b| {
                a.0.cmp(b.0).then_with(|| b.1.partial_cmp(&a.1).unwrap())
            });
            tscores.dedup_by(|a, b| a.0 == b.0);
        }

        for (target_id, target_score) in tscores {
            results
                .entry(*target_id)
                .and_modify(|v| *v += target_score)
                .or_insert(target_score);
        }
        break;
    }
    //dbg!(&results);

    if let Some(output) = args.output {
        for (target_id, score) in &results {
            match output.lock() {
                Ok(mut guard) => writeln!(
                    guard,
                    "{} {} {:.7}",
                    args.query_idx + 1,
                    target_id + 1,
                    score
                )?,
                Err(e) => panic!("ouch: {e}"),
            }
        }
    }
    Ok(())
}

// --------------------------------------------------
#[pymodule]
fn process_hits(_py: Python, m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(process_hits_py, m)?)?;

    Ok(())
}
