fun apply(f, x):
  f(x)
end

print(apply(lam(n): n * 2 end, 21))
