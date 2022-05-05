#!/usr/bin/env bash
eval "$(pyenv init -)"
name=`grep -Po '\bname\s*=\s*"\K.*?(?=")' Cargo.toml | head -1 | tr - _`
version=`grep -Po '\bversion\s*=\s*"\K.*?(?=")' Cargo.toml | head -1`

pyenv shell 3.10.4
pip install --upgrade pip
pip install -U pytest
maturin build --release --strip -i python3.10
pip install --force-reinstall ../target/wheels/$name-$version-cp310-cp310-linux_x86_64.whl
python3.10 -m pytest
