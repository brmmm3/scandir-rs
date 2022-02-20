#!/usr/bin/env bash
eval "$(pyenv init -)"
name=`grep -Po '\bname\s*=\s*"\K.*?(?=")' Cargo.toml | head -1 | tr - _`
version=`grep -Po '\bversion\s*=\s*"\K.*?(?=")' Cargo.toml | head -1`

pyenv shell 3.6.15
pip install --upgrade pip
pip install -U pytest
maturin build --release --strip -i python3.6
pip install --force-reinstall build/wheels/$name-$version-cp36-cp36m-manylinux1_x86_64.whl
python3.6 -m pytest

pyenv shell 3.7.12
pip install --upgrade pip
pip install -U pytest
maturin build --release --strip -i python3.7
pip install --force-reinstall build/wheels/$name-$version-cp37-cp37m-manylinux1_x86_64.whl
python3.7 -m pytest

pyenv shell 3.8.12
pip install --upgrade pip
pip install -U pytest
maturin build --release --strip -i python3.8
pip install --force-reinstall build/wheels/$name-$version-cp38-cp38-manylinux1_x86_64.whl
python3.8 -m pytest

pyenv shell 3.9.10
pip install --upgrade pip
pip install -U pytest
maturin build --release --strip -i python3.9
pip install --force-reinstall build/wheels/$name-$version-cp39-cp39-manylinux1_x86_64.whl
python3.9 -m pytest

pyenv shell 3.10.2
pip install --upgrade pip
pip install -U pytest
maturin build --release --strip -i python3.10
pip install --force-reinstall build/wheels/$name-$version-cp310-cp310-manylinux1_x86_64.whl
python3.10 -m pytest

pyenv shell --unset
