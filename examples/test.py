
import sys
import time

import scandir_rs as r


if __name__ == "__main__":
    if len(sys.argv) < 2:
        print("Usage: test.py <DirName>")
        sys.exit(1)
    root = sys.argv[1]

    print("Get statistics...")
    t1 = time.time()
    c = r.count(root, metadata_ext=True)
    dt = time.time() - t1
    print(c)
    print("dt =", dt)

    print("Get TOC...")
    t1 = time.time()
    toc = r.toc(root)
    dt = time.time() - t1
    print([(key, len(value)) for key, value in toc.items()])
    for key, value in sorted(toc.items()):
        print(key, value[:3])
    print("dt =", dt)

    print("Get detailed list...")
    t1 = time.time()
    lst = r.list(root, metadata_ext=True)
    dt = time.time() - t1
    print(len(lst))
    for nr, (key, value) in enumerate(lst.items()):
        print(f"{key}: {value}")
        if nr > 2:
            break
    print("dt =", dt)
