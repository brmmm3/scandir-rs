
import sys
import time
from threading import Thread

import scandir_rs as r


class Counter(Thread):

    def __init__(self):
        super().__init__(name="Counter", daemon=True)
        self._cnt = 0
        self._bRun = True

    @property
    def cnt(self):
        return self._cnt

    def stop(self):
        self._bRun = False

    def run(self):
        while self._bRun:
            self._cnt += 1
            time.sleep(0.01)


if __name__ == "__main__":
    if len(sys.argv) < 2:
        print("Usage: test.py <DirName>")
        sys.exit(1)
    root = sys.argv[1]

    print("Get statistics...")
    t1 = time.time()
    thr = Counter()
    thr.start()
    c = r.count(root, metadata_ext=True)
    thr.stop()
    dt = time.time() - t1
    print(c)
    print(f"dt={dt:.3f}  cnt={thr.cnt}")

    print("\nGet TOC...")
    t1 = time.time()
    thr = Counter()
    thr.start()
    toc = r.toc(root)
    thr.stop()
    dt = time.time() - t1
    print("KEY -> CNT:", [(key, len(value)) for key, value in toc.items()])
    for key, value in sorted(toc.items()):
        print("KEY -> 3 VALUES:", key, value[:3])
    print(f"dt={dt:.3f}  cnt={thr.cnt}")

    print("\nGet detailed list...")
    t1 = time.time()
    thr = Counter()
    thr.start()
    lst = r.entries(root, metadata_ext=True)
    thr.stop()
    dt = time.time() - t1
    print("CNT:", len(lst))
    for nr, (key, value) in enumerate(lst.items()):
        print(f"KEY -> VALUE: {key}: {value}")
        if nr > 2:
            break
    print(f"dt={dt:.3f}  cnt={thr.cnt}")
