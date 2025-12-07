fun power(base, exp):
  if exp == 0:
    1
  else:
    base * power(base, exp - 1)
  end
end

print(power(2, 10))
