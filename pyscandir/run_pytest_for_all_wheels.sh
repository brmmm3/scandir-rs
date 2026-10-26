#!/usr/bin/env bash
eval "$(pyenv init -)"

name=`grep -Po '\bname\s*=\s*"\K.*?(?=")' Cargo.toml | head -1 | tr - _`
version=`grep -Po '\bversion\s*=\s*"\K.*?(?=")' Cargo.toml | head -1`

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

pyenv shell 3.13.15
pip install --upgrade pip
pip install -U pytest
pip install --force-reinstall ../target/wheels/$name-$version-cp313-cp313-linux_x86_64.whl
python3.13 -m pytest

pyenv shell 3.14.5
pip install --upgrade pip
pip install -U pytest
pip install --force-reinstall ../target/wheels/$name-$version-cp314-cp314-linux_x86_64.whl
python3.14 -m pytest

pyenv shell 3.15.2
pip install --upgrade pip
pip install -U pytest
pip install --force-reinstall ../target/wheels/$name-$version-cp315-cp315-linux_x86_64.whl
python3.15 -m pytest

pyenv shell --unset
