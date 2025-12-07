import lists as L

provide: * end

fun testMap():
  nums = [list: 1, 2, 3, 4, 5]
  doubled = L.map(lam(x): x * 2 end, nums)
  L.length(doubled)
end

fun testFilter():
  nums = [list: 1, 2, 3, 4, 5, 6, 7, 8, 9, 10]
  big = L.filter(lam(x): x > 5 end, nums)
  L.length(big)
end

fun testFold():
  nums = [list: 1, 2, 3, 4, 5]
  sum = L.fold(lam(x, acc): x + acc end, 0, nums)
  sum
end

print(testMap())
print(testFilter())
print(testFold())
