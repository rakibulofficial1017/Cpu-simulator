lst = [1, 2, 3, 4, 5]
print([i*i for i in lst])
print(list(
    map(lambda x: x*x, lst)
))