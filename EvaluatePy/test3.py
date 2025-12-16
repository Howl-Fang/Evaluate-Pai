from multiprocessing import Pool
from tqdm import tqdm
import math
import numpy as np
 
def func(x):
    return math.sin(x)+math.cos(x)

if __name__=="__main__":
    with Pool(processes = 20) as pool:
        result = list(tqdm(pool.imap(func, np.linspace(0,2*math.pi,1000)), total=1000))
    print(result)