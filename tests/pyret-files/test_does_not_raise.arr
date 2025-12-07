fun safe-divide(x, y):
  x / y
end

check "test":
  safe-divide(10, 2) does-not-raise ""
end
