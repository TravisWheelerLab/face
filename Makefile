env:
	./mk-env.sh

build:
	maturin develop

smol: build
	./process_hits/test_accumulation.py \
		--min-seq-len 2 \
		--max-seq-len 5 \
		--num-query-seqs 3 \
		--num-target-seqs 5 \
		--num-hits 3 \
		--seed 1

mid: build
	./process_hits/test_accumulation.py \
		--min-seq-len 400 \
		--max-seq-len 600 \
		--num-query-seqs 10 \
		--num-target-seqs 2000 \
		--num-hits 100 \
		--seed 1

mid2: build
	./process_hits/test_accumulation.py \
		--min-seq-len 400 \
		--max-seq-len 600 \
		--num-query-seqs 10 \
		--num-target-seqs 20000 \
		--num-hits 100 \
		--seed 1

biggie: build
	./process_hits/test_accumulation.py \
		--min-seq-len 400 \
		--max-seq-len 600 \
		--num-query-seqs 10 \
		--num-target-seqs 200000 \
		--num-hits 100 \
		--seed 1
