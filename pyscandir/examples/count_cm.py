# Example with context manager

import time

from scandir_rs import Count, ReturnType

with Count("/usr", return_type=ReturnType.Ext) as instance:
    while instance.busy:
        statistics = instance.results()
        # Do something...
        time.sleep(0.1)
    print(instance.results())
    print(instance.to_speedy())
    print(instance.to_bincode())
    print(instance.to_json())
