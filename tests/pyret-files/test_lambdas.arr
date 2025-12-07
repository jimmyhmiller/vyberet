# Test lambda expressions

check "lambda with no arguments":
  f = lam(): 42 end
  f() is 42
end

check "lambda with one argument":
  double = lam(x): x + x end
  double(5) is 10
  double(21) is 42
end

check "lambda with multiple arguments":
  add3 = lam(a, b, c): a + b + c end
  add3(1, 2, 3) is 6
  add3(10, 20, 30) is 60
end

check "lambda returning lambda (closure)":
  make-adder = lam(x): lam(y): x + y end end
  add5 = make-adder(5)
  add5(3) is 8
  add5(10) is 15
end

check "lambda with complex expression body":
  compute = lam(x, y):
    (x * 2) + (y * 3)
  end
  compute(10, 5) is 35
end
