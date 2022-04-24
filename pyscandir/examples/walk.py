# -*- coding: utf-8 -*-

import os

from scandir_rs import Walk, ReturnType

dirName = "C:/Windows" if os.name == 'nt' else "/tmp"

for root, dirs, files in os.walk(dirName):
    print(root, dirs)

print("*** return_type=RETURN_TYPE_WALK:")
for root, dirs, files in Walk(dirName):
    print("#", root)
    print("dirs", dirs)
    print("files", files)


print("\n*** return_type=RETURN_TYPE_EXT:")
for root, dirs, files, symlinks, other, errors in Walk(dirName, return_type=ReturnType.Ext):
    print("#", root)
    print("dirs", dirs)
    print("files", files)
    print("symlinks", symlinks)
    print("other", other)
    print("errors", errors)
