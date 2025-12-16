#Project made by HF

from os import system
from math import sqrt

n=int(input("Evaluate Pai v0.9\nMade by HF\nNumber of Sum:"))
if n<1:
     raise RuntimeError
np=int(n/100)
sum=0
for i in range(1,n+1):
     sum+=((1/i)**2)
     #print(i,end=" ")
     '''
     if ((i%50)==0):
          print(i,end=" ")  #this part isn't as beautiful as the beneath one.
          if ((i%200)==0):
               print()
     '''
     #Part under raise a problem when width of the window is thin. It prints continuous blocks.
     #Still needs a better solution.
pai=sqrt(6*sum)#sqrt() may use ()**0.5 to instead.
print("\nPai ≈",pai)
system("pause")
#print(sqrt(144.0),144.0**0.5)
