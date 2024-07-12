# Processing FAISS hits with Python/Rust

![Face](./images/face.jpg)

## Setup

- Install Maturin (https://www.maturin.rs/tutorial)

- Create/activate Python venv, install deps **`make env`**

- If you make a change to _src/lib.rs_, run **`maturin develop`**

- Run **`make smol`** in current directory to execute test Python script

## Running

The Python program _./scripts/test_accumulation.py_ presents an interface for calling the Resut code in _src/lib.rs_:

```
$ ./process_hits/test_accumulation.py -h
usage: test_accumulation.py [-h] [-m MIN_SEQ_LEN] [-x MAX_SEQ_LEN]
                            [-q NUM_QUERY_SEQS] [-t NUM_TARGET_SEQS]
                            [-n NUM_HITS] [-T NUM_THREADS] [-s Random seed]
                            [-o Output filename]

Run simulation

options:
  -h, --help            show this help message and exit
  -m MIN_SEQ_LEN, --min-seq-len MIN_SEQ_LEN
                        min_seq_len (default: 2)
  -x MAX_SEQ_LEN, --max-seq-len MAX_SEQ_LEN
                        max_seq_len (default: 5)
  -q NUM_QUERY_SEQS, --num-query-seqs NUM_QUERY_SEQS
                        num_query_seqs (default: 3)
  -t NUM_TARGET_SEQS, --num-target-seqs NUM_TARGET_SEQS
                        num_target_seqs (default: 5)
  -n NUM_HITS, --num-hits NUM_HITS
                        num_hits (default: 3)
  -T NUM_THREADS, --num-threads NUM_THREADS
                        num_hits (default: 1)
  -s Random seed, --seed Random seed
  -o Output filename, --outfile Output filename
```

The _Makefile_ gives several shortcuts to run this program for tests, e.g.:

```
$ make smol
...
./process_hits/test_accumulation.py \
		--min-seq-len 2 \
		--max-seq-len 5 \
		--num-query-seqs 3 \
		--num-target-seqs 5 \
		--num-hits 3 \
		--seed 1
Creating data
Performing score accumulation
Created "sim.out" in 0.0012600421905517578 seconds.
```

## Discussion

The `process_hits_py` in _src/lib.rs_ is the main Rust code that receives the following arguments:

```
fn process_hits_py(
    _py: Python,
    scores: PyReadonlyArray2<f32>,
    indices: PyReadonlyArray2<i64>,
    query_start_p: PyReadonlyArray1<i64>,
    target_start_p: PyReadonlyArray1<i64>,
    output_path: String,
    num_threads: usize,
) -> PyResult<()> {
```

The `scores` is a 2D array of `f32` values where each row represents the scores for each residue of the query sequences.
In the following output from the `smol` run previously, the program is run with 3 query sequences that in total comprised 7 amino acids.
Run with `--num-hits 3` means each AA has three scores:

```
[[0.18626021, 0.34556073, 0.39676747],
 [0.53881675, 0.41919452, 0.6852195],
 [0.20445225, 0.87811744, 0.027387593],
 [0.6704675, 0.4173048, 0.55868983],
 [0.14038694, 0.19810149, 0.8007446],
 [0.9682616, 0.31342417, 0.6923226],
 [0.87638915, 0.89460665, 0.08504421]]
, shape=[7, 3], strides=[3, 1], layout=Cc (0x5), const ndim=2
```

The `query_start_p` has the starting positions of the three query sequences.
In the following, we know the first sequences goes 0-1, the second 2-3, and the third starts at 4 and goes to the end:

```
[0, 2, 4], shape=[3], strides=[1], layout=CFcf (0xf), const ndim=1
```


Each cell in `scores` has a partner in the `indices` array, which is also 2D following the same structure:

```
[[3, 10, 9],
 [8, 7, 3],
 [6, 5, 1],
 [9, 3, 4],
 [8, 1, 11],
 [12, 10, 4],
 [0, 3, 9]]
, shape=[7, 3], strides=[3, 1], layout=Cc (0x5), const ndim=2
```

So we know the scores belong to the following queries:

```
[[0.18626021, 0.34556073, 0.39676747],  < Query 1
[[3,          10,         9],           < Targets

 [0.53881675, 0.41919452, 0.6852195],   < Query 1
 [8,          7,          3],           < Targets

 [0.20445225, 0.87811744, 0.027387593], < Query 2
 [6,          5,          1],           < Targets

 [0.6704675, 0.4173048, 0.55868983],    < Query 2
 [9,         3,         4],             < Targets

 [0.14038694, 0.19810149, 0.8007446],   < Query 3
 [8,          1,          11],          < Targets

 [0.9682616, 0.31342417, 0.6923226],    < Query 3
 [12,        10,         4],            < Targets

 [0.87638915, 0.89460665, 0.08504421]]  < Query 3
 [0,          3,          9]]           < Targets
```

The integet values in `targets` corresponds to a target AA position where all the target sequences can be thought of as existing sequentially.
The `target_start_p` array contains the start positions of the target sequences:

```
[0, 3, 5, 7, 10]
```

We use this to match the `targets` to their sequence.
For example, the value `8` is the eighth AA and falls in the fourth target sequence (offset 3):

```
 0  1  2  3  4   < target_id
[0, 3, 5, 7, 10] < target_starts
           ^ 8 goes here
```

For each query, the `run` function iterates through each position of the query sequence.
First, it gathers the `qscores`:

```
[ 0.18626021, 0.34556073, 0.39676747 ]
```

Next, the raw `qindices` are converted to target IDs using a binary search of the `targets` array:

```
index   target
3       1
10      4
9       3
```

The targets and scores are zipped into a list of tuples (`tscores`):

```
[
    (
        1,
        0.18626021,
    ),
    (
        4,
        0.34556073,
    ),
    (
        3,
        0.39676747,
    ),
]
```

There is an optional step to deduplicate the `tscores` to take only the highest score for each target ID.
Finally, a `results` hashmap is updated to accumulate the score for each target.
In the case of the first query sequence, it hits four different targets to produce the following sums:

```
query_idx = 0
tscores q_1 = [(1, 0.18626021), (4, 0.34556073), (3, 0.39676747)]
tscores q_2 = [(3, 0.53881675), (3, 0.41919452), (1, 0.6852195)]
results     = {4: 0.34556073, 3: 1.3547788, 1: 0.87147975}
```

At the moment, the results are printed to a file where the query and target IDs are incremented to make them one-based values:

```
1 5 0.3455607
1 4 1.3547788
1 2 0.8714797
2 1 0.0273876
2 3 1.0825697
2 4 0.6704675
2 2 0.9759946
3 4 0.2254311
3 1 1.0744907
3 5 2.0824304
3 2 1.5869293
```

Eventually this data will likely be passed to the next process.

## Testing

Run **`cargo test`** to ensure the program produces the correct output.

## Authors

* Daniel Olson <daniel.olson@umconnect.umt.edu>
* Ken Youens-Clark <kyclark@arizona.edu>
