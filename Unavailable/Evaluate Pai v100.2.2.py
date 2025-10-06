from decimal import Decimal, getcontext
from math import factorial
from tkinter import *
from time import perf_counter

def calc_pi(precision):
    getcontext().prec = precision + 1
    pi = Decimal(0)
    for k in range(precision):
        pi += (Decimal(-1)**k * Decimal(factorial(6*k)) * (13591409 + 545140134*k)) / (factorial(3*k) * (factorial(k)**3) * (640320**(3*k + Decimal("1.5"))))
    pi = pi**(-1) * 12
    return pi

def update_result():
    precision = int(entry.get())
    result_label.config(text=str(calc_pi(precision)))

if __name__=="__main__":
    root = Tk()
    root.title("Calculate Pi")

    frame = Frame(root)
    frame.pack()

    label = Label(frame, text="Precision:")
    label.pack(side=LEFT)

    entry = Entry(frame)
    entry.insert(0, "100")
    entry.pack(side=LEFT)

    button = Button(frame, text="Calculate", command=update_result)
    button.pack(side=LEFT)

    result_label = Label(root)
    result_label.pack()

    root.mainloop()