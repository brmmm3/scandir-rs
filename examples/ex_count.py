# -*- coding: utf-8 -*-

import time
import sys

import scandir_rs as scandir


print(scandir.count.Count("~/workspace/test0").collect())
print(scandir.count.Count("~/workspace/test0", extended=True).collect())
