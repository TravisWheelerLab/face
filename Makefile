smol: build
	./process_hits/test_accumulation.py \
		--min-seq-len 2 \
		--max-seq-len 5 \
		--num-query-seqs 2 \
		--num-target-seqs 5 \
		--num-hits 3

env:
	python3 -m venv .venv && source .venv/bin/activate

build:
	maturin develop

test:
	./process_hits/test_accumulation.py 400 600 10000 20000 100
