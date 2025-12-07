fun classify(x):
  if x < 0:
    -1
  else if x == 0:
    0
  else if x < 10:
    1
  else:
    2
  end
end

print(classify(-5))
print(classify(0))
print(classify(3))
print(classify(42))
