pyenv shell 3.10.4
pip install --upgrade pip
pip install -U pytest
maturin build --release --strip -i python3.10
pip install --force-reinstall ../target/wheels/scandir_rs-2.0.3-cp310-none-win_amd64.whl
python3.10 -m pytest
