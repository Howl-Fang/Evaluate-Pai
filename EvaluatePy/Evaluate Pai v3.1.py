#Project made by HF

from os import system, cpu_count
from decimal import Decimal, getcontext
from time import perf_counter
from multiprocessing import Pool

getcontext().prec = 100 #define accuracy
ThreadsN = cpu_count()+5 if cpu_count()!=None else 30 #define the number of threads
#StepsN = 10000 #define the number of steps in one piece of process. Attention to make sure that the value was exactly divided.

def sumup(a):
     t,steps=a
     a=Decimal(0)
     for v in range(t,t+steps):#A hidden mistake may be raised here for some last items bigger than "n".
          a+=Decimal((-1)**(v+1))/Decimal(2*v-1)
     return a

if __name__=="__main__":

     n=int(input("Evaluate Pai v3.1\nMade by HF\nPower of items: ₁₀"))
     perf=perf_counter()
     if n<=1:
          raise ValueError
     n=int(10**n)
     StepsN=int(n/100)
     #np=int(n/100)
     #np=1 if np==0 else np
     sum=Decimal(0)
     Threads=Pool(processes=ThreadsN)
     Input=[(v,StepsN) for v in range(1,n,StepsN)]
     Output=Threads.map(func=sumup,iterable=Input)
     # Here, function map stops the main function until the result is ready.
     for v in Output:
          sum+=v
     sum+=Decimal("0.5")*Decimal((-1)**n)/Decimal(2*n+1)
     pai=sum*4
     print("\nPai ≈",pai,"\nTime spent:",format(perf_counter()-perf,".4f")+"s")#4 num left
     del Input,Output,sum,Threads
     system("pause")
     #print(sqrt(144.0),144.0**0.5)
