# -*- coding: utf-8 -*-

import os

from scandir_rs import Count

dirName = "C:/Windows" if os.name == "nt" else "/usr"
print(Count(dirName).collect())
# Output is something like:
# Statistics { dirs: 76923, files: 648585, slinks: 48089,
#              hlinks: 0, devices: 0, pipes: 0, size: 0, usage: 0,
#              errors: [], duration: 1.07 }
