import random
import threading
from tkinter import *

def monte_carlo_pi(precision):
    circle_points = 0
    square_points = 0

    for _ in range(precision):
        rand_x = random.uniform(-1, 1)
        rand_y = random.uniform(-1, 1)

        origin_dist = rand_x**2 + rand_y**2

        if origin_dist <= 1:
            circle_points += 1

        square_points += 1

    pi = 4 * circle_points / square_points
    return pi

def calc_pi_thread(precision, results):
    results.append(monte_carlo_pi(precision))

def calc_pi(precision):
    threads = []
    results = []
    for _ in range(8):
        t = threading.Thread(target=calc_pi_thread, args=(precision//8, results))
        t.start()
        threads.append(t)

    for t in threads:
        t.join()

    return sum(results) / len(results)

def update_result():
    precision = int(entry.get())
    result_label.config(text=str(calc_pi(precision)))

root = Tk()
root.title("Calculate Pi")

frame = Frame(root)
frame.pack()

label = Label(frame, text="Precision:")
label.pack(side=LEFT)

entry = Entry(frame)
entry.insert(0, "100000")
entry.pack(side=LEFT)

button = Button(frame, text="Calculate", command=update_result)
button.pack(side=LEFT)

result_label = Label(root)
result_label.pack()

root.mainloop()