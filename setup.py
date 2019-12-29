#!/usr/bin/env python3
# coding: utf-8

import setuptools
import setuptools_rust as rust


setuptools.setup(
    name="scandir-rs",
    version="0.3",
    author="Martin Bammer",
    # Find all inplace extensions
    rust_extensions=rust.find_rust_extensions(
        binding=rust.Binding.PyO3, strip=rust.Strip.Debug
    ),
    # specify setup dependencies
    setup_requires=["setuptools", "setuptools_rust"],
    # rust extensions are not zip safe, just like C-extensions.
    zip_safe=False,
)
