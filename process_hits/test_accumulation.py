#!/usr/bin/env python3
"""
Docs here
"""

import argparse
import numpy as np
import process_hits
from time import time
from typing import NamedTuple


class Args(NamedTuple):
    """Command-line arguments"""

    min_seq_len: int
    max_seq_len: int
    num_query_seqs: int
    num_target_seqs: int
    num_hits: int


# --------------------------------------------------
def get_args() -> Args:
    """Get command-line arguments"""

    parser = argparse.ArgumentParser(
        description="Rock the Casbah",
        formatter_class=argparse.ArgumentDefaultsHelpFormatter,
    )

    parser.add_argument(
        "-m", "--min_seq_len", metavar="MIN_SEQ_LEN", type=int, help="min_seq_len"
    )

    parser.add_argument(
        "-x", "--max_seq_len", metavar="MAX_SEQ_LEN", type=int, help="max_seq_len"
    )

    parser.add_argument(
        "-q",
        "--num_query_seqs",
        metavar="NUM_QUERY_SEQS",
        type=int,
        help="num_query_seqs",
    )

    parser.add_argument(
        "-t",
        "--num_target_seqs",
        metavar="NUM_TARGET_SEQS",
        type=int,
        help="num_target_seqs",
    )

    parser.add_argument(
        "-n", "--num_hits", metavar="NUM_HITS", type=int, help="num_hits"
    )

    args = parser.parse_args()

    return Args(
        args.min_seq_len,
        args.max_seq_len,
        args.num_query_seqs,
        args.num_target_seqs,
        args.num_hits,
    )


# --------------------------------------------------
def main() -> None:
    """Make a jazz noise here"""

    args = get_args()

    print("Creating data")

    # Create the sequence data
    query_sequence_lengths = np.random.randint(
        args.min_seq_len, args.max_seq_len, size=args.num_query_seqs
    )
    target_sequence_lengths = np.random.randint(
        args.min_seq_len, args.max_seq_len, size=args.num_target_seqs
    )

    num_query_embeddings = np.sum(query_sequence_lengths)
    num_target_embeddings = np.sum(target_sequence_lengths)

    query_embeddingid_to_seqid = np.zeros(
        (num_query_embeddings,), dtype=np.int32
    )
    target_embeddingid_to_seqid = np.zeros(
        (num_target_embeddings,), dtype=np.int32
    )

    start = 0
    for i in range(args.num_query_seqs):
        end = start + query_sequence_lengths[i]
        query_embeddingid_to_seqid[start:end] = i
        start = end

    start = 0
    for i in range(args.num_target_seqs):
        end = start + target_sequence_lengths[i]
        target_embeddingid_to_seqid[start:end] = i
        start = end

    # Create faux FAISS search results
    test_scores = np.random.rand(num_query_embeddings, args.num_hits).astype(
        np.float32
    )

    test_indices = np.random.randint(
        0,
        num_target_embeddings,
        size=(num_query_embeddings, args.num_hits),
        dtype=np.int64,
    )

    print("Performing score accumulation")
    start_time = time()
    process_hits.process_hits_py(
        test_scores,
        test_indices,
        query_embeddingid_to_seqid,
        target_embeddingid_to_seqid,
        "./test.out",
    )
    end_time = time()
    print("Done. Time taken: {} seconds".format(end_time - start_time))


# --------------------------------------------------
if __name__ == "__main__":
    main()
