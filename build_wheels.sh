#!/usr/bin/env bash
eval "$(pyenv init -)"
pyenv shell 3.6.11
pip install -U pytest
maturin build --release --strip -i python3.6
pip install -U build/temp.linux-x86_64-3.7/scandir-rs/wheels/scandir_rs-0.9.2-cp36-cp36m-manylinux1_x86_64.whl
python3.6 -m pytest
pyenv shell 3.7.8
pip install -U pytest
maturin build --release --strip -i python3.7
pip install -U build/temp.linux-x86_64-3.7/scandir-rs/wheels/scandir_rs-0.9.2-cp37-cp37m-manylinux1_x86_64.whl
python3.7 -m pytest
pyenv shell 3.8.5
pip install -U pytest
maturin build --release --strip -i python3.8
pip install -U build/temp.linux-x86_64-3.7/scandir-rs/wheels/scandir_rs-0.9.2-cp38-cp38-manylinux1_x86_64.whl
python3.8 -m pytest
pyenv shell 3.9.0b5
pip install -U pytest
maturin build --release --strip -i python3.9
pip install -U build/temp.linux-x86_64-3.7/scandir-rs/wheels/scandir_rs-0.9.2-cp39-cp39-manylinux1_x86_64.whl
python3.9 -m pytest
pyenv shell --unset
