#Project made by HF

from os import system
from decimal import Decimal, getcontext
from time import perf_counter

getcontext().prec=100 #define accuracy

n=int(input("Evaluate Pai v2.1\nMade by HF\nNumber of items to sum:"))
perf=perf_counter()
if n<1:
     raise Exception
np=int(n/100)
np=1 if np==0 else np
sum=Decimal(0)
for i in range(1,n+1):
     sum+=Decimal((-1)**(i+1))/Decimal(2*i-1)
     #print(i,end=" ")
     '''
     if ((i%50)==0):
          print(i,end=" ")  #this part isn't as beautiful as the beneath one.
          if ((i%200)==0):
               print()
     '''
     #Part under raise a problem when width of the window is thin. It prints continuous of blocks.
     #Still needs a better solution.
     if (np!=0)&(i%np==0):#put (np!=0) in the front for avoiding n smaller than 100 which makes np to be zero.
          print("\r",int(i/n*100),"%",end="",sep="") #"\r" is used to clear this line.
sum+=Decimal("0.5")*Decimal((-1)**n)/Decimal(2*n+1)
pai=sum*4
print("\r100%","▍"*100,"\nPai ≈",pai,"\nTime spent:",format(perf_counter()-perf,".4f"),"s")#4 num left
system("pause")
#print(sqrt(144.0),144.0**0.5)
