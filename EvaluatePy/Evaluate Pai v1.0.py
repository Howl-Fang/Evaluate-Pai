#Project made by HF

from os import system
from math import sqrt

n=int(input("Evaluate Pai v1.0\nMade by HF\nNumber of Sum:"))
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
     if (np!=0)&(i%np==0):#put (np!=0) in the front for avoiding n smaller than 100 which makes np to be zero.
          print("\r",int(i/n*100),"%","▍"*int((int(i)/np))," ",end="") #"\r" is used to clear this line.
pai=sqrt(6*sum)#sqrt() may use ()**0.5 to instead.
print("\r100%","▍"*100,"\nPai ≈",pai)
system("pause")
#print(sqrt(144.0),144.0**0.5)
