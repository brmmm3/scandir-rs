# -*- coding: utf-8 -*-

import os
import time
import sys

import scandir_rs as scandir

dirName = "C:/Windows" if os.name == 'nt' else "/tmp"

print("*** return_type=RETURN_TYPE_WALK:")
for root, dirs, files in scandir.walk.Walk(dirName,
                                           return_type=scandir.RETURN_TYPE_WALK):
    print("#", root)
    print("dirs", dirs)
    print("files", files)


print("\n*** return_type=RETURN_TYPE_EXT:")
for root, dirs, files, symlinks, other, errors in scandir.walk.Walk(dirName,
                                                                    return_type=scandir.RETURN_TYPE_EXT):
    print("#", root)
    print("dirs", dirs)
    print("files", files)
    print("symlinks", symlinks)
    print("other", other)
    print("errors", errors)
