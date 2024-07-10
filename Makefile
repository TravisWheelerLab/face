smol: build
	./process_hits/test_accumulation.py \
		--min_seq_len 2 \
		--max_seq_len 5 \
		--num_query_seqs 2 \
		--num_target_seqs 5 \
		--num_hits 3

build:
	maturin develop

test:
	./process_hits/test_accumulation.py 400 600 10000 20000 100
