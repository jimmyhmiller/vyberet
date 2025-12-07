# Test function definitions

check "simple function":
  fun double(x):
    x * 2
  end
  double(5) is 10
  double(21) is 42
end

check "function with multiple parameters":
  fun add(a, b):
    a + b
  end
  add(10, 5) is 15
  add(100, 200) is 300
end

check "function calling function":
  fun square(x):
    x * x
  end
  fun sum-of-squares(a, b):
    square(a) + square(b)
  end
  sum-of-squares(3, 4) is 25
end

check "recursive function":
  fun factorial(n):
    if n <= 1:
      1
    else:
      n * factorial(n - 1)
    end
  end
  factorial(0) is 1
  factorial(1) is 1
  factorial(5) is 120
end

check "function with local variables":
  fun compute(x, y):
    temp = x * 2
    result = temp + y
    result
  end
  compute(5, 3) is 13
end
