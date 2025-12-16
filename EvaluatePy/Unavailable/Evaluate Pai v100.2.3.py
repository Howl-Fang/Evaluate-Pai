# 导入所需的模块
import multiprocessing as mp
from decimal import Decimal, getcontext

# 设置Decimal的精度为100位
getcontext().prec = 100


# 定义一个函数，用于计算泰勒展开中每一项的值
def taylor_term(n):
    # 计算(-1)^n / (2n+1)
    return Decimal((-1) ** n) / Decimal(2 * n + 1)


# 定义一个函数，用于计算泰勒展开中前N项的和
def taylor_sum(N):
    # 创建一个进程池，使用所有可用的核心
    pool = mp.Pool()
    # 使用map方法将taylor_term函数应用到0到N-1之间的所有整数上，并返回一个列表
    terms = pool.map(taylor_term, range(N))
    # 关闭进程池并释放资源
    pool.close()
    pool.join()
    # 计算列表中所有元素的和，并乘以4，得到圆周率的近似值
    pi = sum(terms) * 4
    # 返回圆周率的近似值
    return pi


# 定义一个函数，用于创建图形化界面，并显示结果
def gui():
    # 导入PySimpleGUI模块[^6^][6]
    import PySimpleGUI as sg

    # 设置窗口布局，包括两个文本框和两个按钮
    layout = [
        [sg.Text("请输入泰勒展开项数："), sg.Input(key="N")],
        [sg.Button("计算"), sg.Button("退出")],
        [sg.Text("圆周率近似值为："), sg.Text(size=(40, 1), key="pi")]
    ]

    # 创建窗口对象，并设置标题为"求圆周率"
    window = sg.Window("求圆周率", layout)

    # 进入事件循环，等待用户操作
    while True:
        event, values = window.read()  # 读取事件和输入值

        if event == "退出" or event == sg.WIN_CLOSED:  # 如果用户点击退出按钮或关闭窗口，则退出循环
            break

        if event == "计算":  # 如果用户点击计算按钮，则执行以下操作

            try:
                N = int(values["N"])  # 尝试将输入值转换为整数

                if N > 0:  # 如果输入值是正整数，则调用taylor_sum函数计算圆周率，并显示结果

                    pi = taylor_sum(N)
                    window["pi"].update(pi)

                else:  # 如果输入值不是正整数，则显示错误信息

                    window["pi"].update("请输入正整数！")

            except ValueError:  # 如果输入值不能转换为整数，则显示错误信息

                window["pi"].update("请输入有效数字！")

        # 关闭窗口并结束程序
    window.close()


# 调用gui函数运行程序
gui()