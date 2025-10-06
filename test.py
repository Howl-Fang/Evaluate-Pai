class juesei():
    def __init__(self):
        self.name == "圆圆"
        self.nengli = 100


class wuqi_jian():
    def __init__(self):
        self.nengli = 1000

class guai():
    def __init__(self):
        self.nengli = 10000


class animal:
    def __init__(self):
        self.tmp222 = 3

    def test1(self):
        print(11111111111)



class dog(animal):
    def __init__(self, tmp):
        # super(dog, self).__init__()
        if tmp == 1:
            self.color = "heise"
            self.age = "5"
            self.wangwang()
        else:
            self.color = "baise"
            self.age = "1"
            self.wangwang()

    def run(self):
        pass

    def eat(self, key1, key2=2, key3=3):
        pass

    def wangwang(self):
        pass

    def test1(self):
        print(22222222222)


gou1 = dog()
print(gou1.tmp222)


