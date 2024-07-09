#!/usr/bin/env python3

import argparse
import faiss
import pickle
import sys
import numpy as np
import process_hits
from Bio import SeqIO
from typing import NamedTuple, Optional, TextIO


class Args(NamedTuple):
    """Command-line arguments"""

    query_path: str
    target_path: str
    output_path: TextIO
    gpu: bool
    start: Optional[int]
    end: Optional[int]


# --------------------------------------------------
def get_args() -> Args:
    """Get command-line arguments"""

    # Usage: faiss_search.py <query path:softmasked fasta> <target
    # path:softmasked fasta> <output path> <gpu|cpu> [embedding start, end]")

    parser = argparse.ArgumentParser(
        description="Rock the Casbah",
        formatter_class=argparse.ArgumentDefaultsHelpFormatter,
    )

    parser.add_argument(
        "-q",
        "--query",
        help="Query FASTA file",
        metavar="FILE",
        type=argparse.FileType("rt"),
        required=True,
    )

    parser.add_argument(
        "-t",
        "--target",
        help="Target FASTA file",
        metavar="FILE",
        type=argparse.FileType("rt"),
        required=True,
    )

    parser.add_argument(
        "-o",
        "--output",
        help="Output file",
        metavar="FILE",
        type=argparse.FileType("rt"),
        default="out.txt",
    )

    parser.add_argument(
        "-s", "--start", help="Embedding start", metavar="int", type=int
    )

    parser.add_argument(
        "-e", "--end", help="Embedding end", metavar="int", type=int
    )

    parser.add_argument("-g", "--gpu", help="GPU", action="store_true")

    args = parser.parse_args()
    args.query_path.close()
    args.target_path.close()
    args.output_path.close()

    return Args(
        query_path=args.query.name,
        target_path=args.target.name,
        output_path=args.output,
        gpu=args.gpu,
        start=args.start,
        end=args.end,
    )


# --------------------------------------------------
def main() -> None:
    """Make a jazz noise here"""

    args = get_args()
    query_path = args.query_path
    soft_mask_query_path = None

    if ":" in query_path:
        query_path, soft_mask_query_path = query_path.split(":")

    target_path = args.target_path
    soft_mask_target_path = None
    if ":" in target_path:
        target_path, soft_mask_target_path = target_path.split(":")

    start_index = args.start
    end_index = args.end
    index_str = "IVF5000,PQ32"

    print(f"Reading query file: {query_path}")
    query_data = read_embedding_pickle(
        query_path,
        soft_mask_query_path,
        start_index,
        end_index,
        transpose=True,
    )

    print(f"Reading target file: {target_path}")
    target_data = read_embedding_pickle(
        target_path,
        soft_mask_target_path,
        start_index,
        end_index,
        transpose=True,
    )

    print("Preparing query data")
    query_seq_ids, query_embeddings, query_lengths = prepare_embedding_data(
        query_data
    )

    print("Preparing target data")
    target_seq_ids, target_embeddings, target_lengths = (
        prepare_embedding_data(target_data)
    )

    query_seq_ids = np.array(query_seq_ids, dtype=np.int32)
    target_seq_ids = np.array(target_seq_ids, dtype=np.int32)

    print("Creating index from target data")
    target_index = faiss.index_factory(
        target_embeddings.shape[-1], index_str, faiss.METRIC_INNER_PRODUCT
    )

    if args.gpu:
        gpu_resource = faiss.StandardGpuResources()
        target_index = faiss.index_cpu_to_gpu(gpu_resource, 0, target_index)

    print("Training index")
    target_index.train(target_embeddings)

    print("Adding indices")
    target_index.add(target_embeddings)

    print("Searching")
    target_index.nprobe = 150
    res_scores, res_indices = batched_search(
        target_index, query_embeddings, k=100
    )

    res_scores = res_scores - 0.45
    print("Processing and outing hits")

    # del query_embeddings
    # del target_embeddings
    # del target_index

    # query_embeddings = None
    # target_embeddings = None
    # target_index = None
    # process_hits(res_scores, res_indices, query_seq_ids, target_seq_ids, output_path)

    process_hits.process_hits_py(
        np.float32(res_scores),
        np.int64(res_indices),
        query_seq_ids,
        target_seq_ids,
        args.output_path,
    )


# --------------------------------------------------
def read_embedding_pickle(
    file_path, soft_mask_path, start_index, end_index, transpose=False
):
    """ Does something """

    with open(file_path, "rb") as file:
        old_data = pickle.load(file)

    data = []
    for i in range(len(old_data)):
        name = str(old_data[i][0])
        if "-" in name:
            name = name[: name.find("-")]
        if transpose:
            data.append((name, old_data[i][1].T))
        else:
            data.append((name, old_data[i][1]))

    old_data = None
    data = sorted(data, key=lambda x: int(x[0]))
    if start_index is not None or end_index is not None:
        print("Slicing embeddings")
        for i in range(len(data)):
            data[i] = (data[i][0], data[i][1][start_index:end_index])

    if soft_mask_path is not None:
        print("Reducing embeddings based on softmask")
        with open(soft_mask_path, "r") as file:
            sequences = list(SeqIO.parse(file, "fasta"))
            # sequences = sorted(sequences, key=lambda x:str(x.id))

            for i, seq in enumerate(sequences):
                # print(i, data[i][0], seq.id, data[i][1].shape, len(str(seq.seq)))
                name = str(seq.id)
                if "-" in name:
                    name = name[: name.find("-")]
                if name != str(data[i][0]):
                    print("Masks and data not lining up!", seq.id, data[i][0])
                    exit(-1)
                mask = np.array(
                    [True if c.isupper() else False for c in str(seq.seq)],
                    dtype=bool,
                )
                data[i] = (data[i][0], data[i][1][mask].copy())

    return data


# --------------------------------------------------
def prepare_embedding_data(data):
    """ Does something """

    names = []
    all_embeddings = []
    lengths = []

    for d in data:
        seqid = int(d[0]) - 1
        embedding = d[1]  # .T

        embedding = embedding / np.linalg.norm(embedding, axis=1)[:, None]
        names.extend([seqid for _ in range(embedding.shape[0])])
        all_embeddings.append(embedding)
        lengths.append(embedding.shape[0])

    return names, np.concatenate(all_embeddings, axis=0), lengths


# --------------------------------------------------
def batched_search(target_index, query_embeddings, k=1000, batch_size=512):
    """ Does something """

    num_queries = query_embeddings.shape[0]
    num_batches = (
        num_queries + batch_size - 1
    ) // batch_size  # Calculate the number of batches

    all_scores = []
    all_indices = []

    for i in range(num_batches):
        start_idx = i * batch_size
        end_idx = min((i + 1) * batch_size, num_queries)
        batch_query_embeddings = query_embeddings[start_idx:end_idx]

        res_scores, res_indices = target_index.search(
            batch_query_embeddings, k
        )

        all_scores.append(res_scores)
        all_indices.append(res_indices)

    # Combine results from all batches
    combined_scores = np.vstack(all_scores)
    combined_indices = np.vstack(all_indices)

    return combined_scores, combined_indices


# --------------------------------------------------
if __name__ == "__main__":
    main()
