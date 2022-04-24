#!/usr/bin/env bash
eval "$(pyenv init -)"
name=`grep -Po '\bname\s*=\s*"\K.*?(?=")' Cargo.toml | head -1 | tr - _`
version=`grep -Po '\bversion\s*=\s*"\K.*?(?=")' Cargo.toml | head -1`

pyenv shell 3.8.12
pip install --upgrade pip
pip install -U pytest
maturin build --release --strip -i python3.8
pip install --force-reinstall build/wheels/$name-$version-cp38-cp38-manylinux1_x86_64.whl
python3.8 -m pytest

pyenv shell --unset
