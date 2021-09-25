#!/usr/bin/env bash
eval "$(pyenv init -)"
version=`grep -Po '\bversion\s*=\s*"\K.*?(?=")' Cargo.toml | head -1`
pyenv shell 3.7.10
pip install -U pytest
maturin build --release --strip -i python3.7
pip install -U build/temp.linux-x86_64-3.7/scandir-rs/wheels/scandir_rs-$version-cp37-cp37m-manylinux1_x86_64.whl
python3.7 -m pytest
pyenv shell 3.8.10
pip install -U pytest
maturin build --release --strip -i python3.8
pip install -U build/temp.linux-x86_64-3.7/scandir-rs/wheels/scandir_rs-$version-cp38-cp38-manylinux1_x86_64.whl
python3.8 -m pytest
pyenv shell 3.9.0
pip install -U pytest
maturin build --release --strip -i python3.9
pip install -U build/temp.linux-x86_64-3.7/scandir-rs/wheels/scandir_rs-$version-cp39-cp39-manylinux1_x86_64.whl
python3.9 -m pytest
pyenv shell --unset
