#!/usr/bin/env bash
eval "$(pyenv init -)"
name=`grep -Po '\bname\s*=\s*"\K.*?(?=")' Cargo.toml | head -1 | tr - _`
version=`grep -Po '\bversion\s*=\s*"\K.*?(?=")' Cargo.toml | head -1`

pyenv shell 3.8.10
pip install --upgrade pip
pip install -U pytest
maturin build --release --strip -i python3.8
pip install --force-reinstall ../tareget/wheels/$name-$version-cp38-cp38-linux_x86_64.whl
python3.8 -m pytest
