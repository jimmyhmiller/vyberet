fun divide(x, y):
  if y == 0:
    raise("Division by zero")
  else:
    x / y
  end
end

check "raises":
  divide(10, 0) raises "Division by zero"
end
