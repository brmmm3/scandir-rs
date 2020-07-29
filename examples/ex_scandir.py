# -*- coding: utf-8 -*-

import os
import time
import sys

import scandir_rs as scandir


dirName = "C:/Windows/appcompat" if os.name == 'nt' else "/tmp"

print("*** return_type=RETURN_TYPE_FAST:")
for pathName, dirEntry in scandir.scandir.Scandir(dirName,
                                                  return_type=scandir.RETURN_TYPE_FAST):
    print("#", pathName, dirEntry)


print("*** return_type=RETURN_TYPE_BASE:")
for pathName, dirEntry in scandir.scandir.Scandir(dirName,
                                                  return_type=scandir.RETURN_TYPE_BASE):
    print("#", pathName, dirEntry)


print("*** return_type=RETURN_TYPE_EXT:")
for pathName, dirEntry in scandir.scandir.Scandir(dirName,
                                                  return_type=scandir.RETURN_TYPE_EXT):
    print("#", pathName, dirEntry)


print("*** return_type=RETURN_TYPE_FULL:")
for pathName, dirEntry in scandir.scandir.Scandir(dirName,
                                                  file_include=["*.txt"],
                                                  return_type=scandir.RETURN_TYPE_FULL):
    print("#", pathName, dirEntry)
