import multiprocessing
from tqdm import tqdm
import time, random
from math import cos, sin


def task_deal():
    time.sleep(random.random() * 0.1)
    #for i in range(1,10*t):
    #    a+=cos(sin(cos(sin(random.randint(-t,t)))))
    #return a


if __name__ == '__main__':
    pool = multiprocessing.Pool(multiprocessing.cpu_count())

    param_v = range(int(1e4))
    pbar = tqdm(total=len(param_v))
    #pbar = tqdm(100)
    resy = []
    #output=pool.map(func=task_deal, iterable=param_v)
    for p in param_v:
        res = pool.apply_async(task_deal, callback=lambda _: pbar.update(1))
        resy.append(res)
    pool.close()
    pool.join()
    pbar.close()