fun test(x):
  if x == 0:
    "zero"
  else if x == 1:
    "one"
  else:
    "other"
  end
end

check:
  test(0) is "zero"
  test(1) is "one"
  test(2) is "other"
end
