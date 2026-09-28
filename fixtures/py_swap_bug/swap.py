def swap(a, b):
    a = b
    b = a
    return a, b


if __name__ == "__main__":
    a, b = swap(1, 2)
    print(f"a={a} b={b}")
