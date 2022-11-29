#!/usr/bin/env bash
eval "$(pyenv init -)"

pyenv shell 3.7.15
maturin build --release --strip --sdist
pyenv shell 3.8.15
maturin build --release --strip --sdist
pyenv shell 3.9.15
maturin build --release --strip --sdist
pyenv shell 3.10.8
maturin build --release --strip --sdist
pyenv shell 3.11.0
maturin build --release --strip --sdist
