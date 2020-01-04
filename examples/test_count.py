import time

import scandir_rs as r

C = r.count.Count("~/workspace", metadata_ext=True)

with C:
    while C.busy():
        print(C.statistics)
        time.sleep(0.1)
print("FINISHED")
print(C.statistics)
