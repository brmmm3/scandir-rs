# -*- coding: utf-8 -*-

import time
import sys

import scandir_rs as scandir


print("*** return_type=RETURN_TYPE_WALK:")
for pathName, dirEntry in scandir.scandir.Scandir("~/workspace/test0",
                                                  return_type=scandir.RETURN_TYPE_FAST):
    print("#", pathName, dirEntry)


print("*** return_type=RETURN_TYPE_WALK:")
for pathName, dirEntry in scandir.scandir.Scandir("~/workspace/test0",
                                                  return_type=scandir.RETURN_TYPE_BASE):
    print("#", pathName, dirEntry)


print("*** return_type=RETURN_TYPE_WALK:")
for pathName, dirEntry in scandir.scandir.Scandir("~/workspace/test0",
                                                  return_type=scandir.RETURN_TYPE_EXT):
    print("#", pathName, dirEntry)


print("*** return_type=RETURN_TYPE_WALK:")
for pathName, dirEntry in scandir.scandir.Scandir("~/workspace/test0",
                                                  file_include=["*.txt"],
                                                  return_type=scandir.RETURN_TYPE_FULL):
    print("#", pathName, dirEntry)
