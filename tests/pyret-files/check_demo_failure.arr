fun add(x, y):
  x + y
end

check "addition tests":
  add(2, 3) is 5
  add(10, 5) is 15
  add(1, 1) is 3  # This will fail!
  add(0, 0) is 0
end
