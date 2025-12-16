import math
import decimal
import multiprocessing
from time import perf_counter
from os import system

# 设置decimal模块的精度为100位
decimal.getcontext().prec = 100

# 定义一个函数，计算阶乘
def factorial(n):
  result = 1
  for i in range(1, n + 1):
    result *= i
  return result

# 定义一个函数，使用泰勒级数求圆周率的一部分和
def partial_pi(start, end):
  # start是起始项数，end是结束项数（不包含）
  sum = decimal.Decimal(0) # 记录求和结果，使用decimal类型
  term = decimal.Decimal(0) # 记录每一项的值，使用decimal类型
  sign = (-1)**start # 记录每一项的符号，正负交替
  for k in range(start, end):
    # 根据公式：pi/4 = arctan(1) = sum((-1)^k * (1/(2k+1)))
    term = sign * decimal.Decimal(1) / (2 * k + 1) # 计算第k项的值，使用decimal类型进行运算
    sum += term # 累加到求和结果中
    sign *= -1 # 改变符号
  return sum # 返回部分和

if __name__=="__main__":
    n = int(input("请输入泰勒级数的项数：\n")) # 输入泰勒级数的项数
    perf = perf_counter()

    # 创建一个进程池，设置进程个数为CPU核心个数
    pool = multiprocessing.Pool(multiprocessing.cpu_count())

    # 将泰勒级数分成若干段，并将每段的起始和结束项数作为参数传给partial_pi函数，并行执行，并将结果存入results列表中
    results = [pool.apply_async(partial_pi, args=(i*n//multiprocessing.cpu_count(), (i+1)*n//multiprocessing.cpu_count())) for i in range(multiprocessing.cpu_count())]

    # 关闭进程池，等待所有进程完成任务
    pool.close()
    pool.join()

    # 将results列表中的所有部分和相加得到总和，并乘以4得到圆周率近似值
    pi_value = sum([result.get() for result in results]) * 4

    print("根据泰勒级数求得的圆周率近似值为：\n")
    print(pi_value) # 输出圆周率近似值
    print("\nTime spent:", format(perf_counter() - perf, ".4f") + "s")
    system("pause")