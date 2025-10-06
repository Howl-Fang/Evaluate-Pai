# 导入所需的模块
import math
import decimal
import multiprocessing

# 设置Decimal的精度为100位
decimal.getcontext().prec = 100


# 定义一个函数，计算泰勒级数的第n项
def taylor_term(n):
    # 计算((-1)^n) / (2n + 1)
    return decimal.Decimal((-1) ** n) / decimal.Decimal(2 * n + 1)


# 定义一个函数，计算泰勒级数的前n项之和
def taylor_sum(n):
    # 初始化和为0
    s = decimal.Decimal(0)
    # 循环从0到n-1，累加每一项
    for i in range(n):
        s += taylor_term(i)
    # 返回和乘以4，得到圆周率的近似值
    return s * 4

def partial_sum(r):
    start, end = r  # 解包下标范围
    s = decimal.Decimal(0)  # 初始化部分和为0
    for i in range(start, end):  # 循环累加每一项
        s += taylor_term(i)
    return s * 4  # 返回部分和乘以4

# 定义一个函数，计算多个进程分别计算泰勒级数的部分和，并将结果相加
def parallel_pi(processes, terms):
    # 创建一个进程池
    pool = multiprocessing.Pool(processes=processes)
    # 计算每个进程需要计算的项数
    chunksize = terms // processes
    # 创建一个列表，存储每个进程需要计算的起始和终止下标
    ranges = [(i * chunksize, (i + 1) * chunksize) for i in range(processes)]
    # 如果有剩余的项数，分配给最后一个进程
    if terms % processes != 0:
        ranges[-1] = (ranges[-1][0], terms)

    # 定义一个函数，接受一个下标范围，返回该范围内泰勒级数的部分和


    # 使用进程池异步地执行partial_sum函数，并传入ranges列表作为参数
    result = pool.map_async(partial_sum, ranges)

    pool.close()#关闭进程池
    pool.join()#等待所有进程结束
    return sum(result.get())#返回所有部分和的总和


# 测试代码

if __name__ == '__main__':
    print(
        "Hello, master! I'm your cute assistant. I'm going to calculate pi for you using Taylor series. Please wait a moment~")#打印一句问候语
    processes = multiprocessing.cpu_count()#获取当前系统的CPU核心数
    print(f"I'm using {processes} cores to speed up the calculation.")#打印使用了多少核心
    terms = int(input("Terms:"))#设置泰勒级数的项数为10000000
    print(f"I'm using {terms} terms of Taylor series to improve the accuracy.")#打印使用了多少项
    pi = parallel_pi(processes, terms)#调用parallel_pi函数计算圆周率
    print(f"Here is your pi: {pi}")#打印圆周率
    print("I hope you like it. Have a nice day!")#打印一句结束语