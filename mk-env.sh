#!/usr/bin/env bash
#
# Make Python virtual env and install requirements
#

[[ ! -d .venv ]] && python3 -m venv .venv 
source .venv/bin/activate
python3 -m pip install -U pip -r ./process_hits/requirements.txt
python3 -m pip freeze
