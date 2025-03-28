#!/usr/bin/env bash
eval "$(pyenv init -)"

name=`grep -Po '\bname\s*=\s*"\K.*?(?=")' Cargo.toml | head -1 | tr - _`
version=`grep -Po '\bversion\s*=\s*"\K.*?(?=")' Cargo.toml | head -1`

pyenv shell 3.7.15
pip install --upgrade pip
pip install -U pytest
pip install --force-reinstall ../target/wheels/$name-$version-cp37-cp37m-linux_x86_64.whl
python3.7 -m pytest

pyenv shell 3.8.15
pip install --upgrade pip
pip install -U pytest
pip install --force-reinstall ../target/wheels/$name-$version-cp38-cp38-linux_x86_64.whl
python3.8 -m pytest

pyenv shell 3.9.15
pip install --upgrade pip
pip install -U pytest
pip install --force-reinstall ../target/wheels/$name-$version-cp39-cp39-linux_x86_64.whl
python3.9 -m pytest

pyenv shell 3.10.14
pip install --upgrade pip
pip install -U pytest
pip install --force-reinstall ../target/wheels/$name-$version-cp310-cp310-linux_x86_64.whl
python3.10 -m pytest

pyenv shell 3.11.11
pip install --upgrade pip
pip install -U pytest
pip install --force-reinstall ../target/wheels/$name-$version-cp311-cp311-linux_x86_64.whl
python3.11 -m pytest

pyenv shell 3.12.8
pip install --upgrade pip
pip install -U pytest
pip install --force-reinstall ../target/wheels/$name-$version-cp312-cp312-linux_x86_64.whl
python3.12 -m pytest

pyenv shell 3.13.2
pip install --upgrade pip
pip install -U pytest
pip install --force-reinstall ../target/wheels/$name-$version-cp313-cp313-linux_x86_64.whl
python3.13 -m pytest

pyenv shell --unset
