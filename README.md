# Processing FAISS hits with Python/Rust

![Face](./images/face.jpg)

## Setup

- Install Maturin (https://www.maturin.rs/tutorial)

- Create/activate Python venv, install Maturin

```
python3 -m venv .venv
source .venv/bin/activate
python3 -m pip install -U pip maturin
python3 -m pip freeze
```

- If you make a change to _src/lib.rs_, run **`maturin develop`**

- Run **`make smol`** in current directory to execute test Python script

## Authors

* Daniel Olson <daniel.olson@umconnect.umt.edu>
* Ken Youens-Clark <kyclark@arizona.edu>
