import math
import decimal

# 设置decimal模块的精度为100位
decimal.getcontext().prec = 100

# 定义一个函数，计算阶乘
def factorial(n):
  result = 1
  for i in range(1, n + 1):
    result *= i
  return result

# 定义一个函数，使用泰勒级数求圆周率
def pi(n):
  # n是泰勒级数的项数
  sum = decimal.Decimal(0) # 记录求和结果，使用decimal类型
  term = decimal.Decimal(0) # 记录每一项的值，使用decimal类型
  sign = 1 # 记录每一项的符号，正负交替
  for k in range(n):
    # 根据公式：pi/4 = arctan(1) = sum((-1)^k * (1/(2k+1)))
    term = sign * decimal.Decimal(1) / (2 * k + 1) # 计算第k项的值，使用decimal类型进行运算
    sum += term # 累加到求和结果中
    sign *= -1 # 改变符号
  return sum * 4 # 返回圆周率近似值，乘以4得到pi

n = int(input("请输入泰勒级数的项数：\n")) # 输入泰勒级数的项数

print("根据泰勒级数求得的圆周率近似值为：\n")
print(pi(n)) # 输出圆周率近似值