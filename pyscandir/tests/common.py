
import os
import tempfile


def CreateTempFileTree(dirCnt: int, depthCnt: int, fileCnt: int):
    print(
        f"Create temporary directory with {dirCnt} directories with depth {depthCnt} and {3 * fileCnt} files")
    tempDir = tempfile.TemporaryDirectory(prefix="scandir_rs_")
    for dn in range(dirCnt):
        dirName = f"{tempDir.name}/dir{dn}"
        for depth in range(depthCnt):
            os.makedirs(dirName)
            for fn in range(fileCnt):
                open(f"{dirName}/file{fn}.bin", "wb").close()
                open(f"{dirName}/file{fn}.txt", "wb").close()
                open(f"{dirName}/file{fn}.log", "wb").close()
            dirName = f"{dirName}/dir{depth}"
    return tempDir
