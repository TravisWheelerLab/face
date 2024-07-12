use anyhow::Result;
use assert_cmd::Command;
use pretty_assertions::assert_eq;
use std::fs;
use tempfile::NamedTempFile;

const PRG: &str = "./process_hits/test_accumulation.py";

struct Args {
    min_seq_len: usize,
    max_seq_len: usize,
    num_query_seqs: usize,
    num_target_seqs: usize,
    num_hits: usize,
    seed: usize,
}

// --------------------------------------------------
fn run(args: Args, expected_file: &str) -> Result<()> {
    //let outpath = gen_nonexistent_file();
    let outfile = NamedTempFile::new()?;
    let outpath = &outfile.path().to_str().unwrap();
    let argv = vec![
        "--min-seq-len".to_string(),
        args.min_seq_len.to_string(),
        "--max-seq-len".to_string(),
        args.max_seq_len.to_string(),
        "--num-query-seqs".to_string(),
        args.num_query_seqs.to_string(),
        "--num-target-seqs".to_string(),
        args.num_target_seqs.to_string(),
        "--num-hits".to_string(),
        args.num_hits.to_string(),
        "--seed".to_string(),
        args.seed.to_string(),
        "--outfile".to_string(),
        outpath.to_string(),
    ];

    let output = Command::new(PRG).args(argv).output().unwrap();
    assert!(output.status.success());
    assert!(&outfile.path().exists());

    println!("Reading {outpath}");
    let actual = fs::read_to_string(&outpath)?;
    outfile.close()?;

    let mut actual: Vec<_> = actual.split('\n').collect();
    actual.sort();

    println!("Reading {expected_file}");
    let expected = fs::read_to_string(&expected_file)?;
    let mut expected: Vec<_> = expected.split('\n').collect();
    expected.sort();

    assert_eq!(actual, expected);

    Ok(())
}

// --------------------------------------------------
#[test]
fn test1() -> Result<()> {
    run(
        Args {
            min_seq_len: 2,
            max_seq_len: 5,
            num_query_seqs: 3,
            num_target_seqs: 5,
            num_hits: 3,
            seed: 1,
        },
        "tests/outputs/2.5.3.5.3.1.txt",
    )
}

// --------------------------------------------------
#[test]
fn test2() -> Result<()> {
    run(
        Args {
            min_seq_len: 2,
            max_seq_len: 5,
            num_query_seqs: 3,
            num_target_seqs: 5,
            num_hits: 3,
            seed: 2,
        },
        "tests/outputs/2.5.3.5.3.2.txt",
    )
}

// --------------------------------------------------
#[test]
fn test3() -> Result<()> {
    run(
        Args {
            min_seq_len: 400,
            max_seq_len: 600,
            num_query_seqs: 10,
            num_target_seqs: 2000,
            num_hits: 100,
            seed: 1,
        },
        "tests/outputs/400.600.10.2000.100.1.txt",
    )
}
