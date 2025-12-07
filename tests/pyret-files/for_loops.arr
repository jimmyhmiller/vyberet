# Test for loop expressions

# Test 1: for map - apply function to each element
nums = [list: 1, 2, 3, 4, 5]
doubled = for map(x from nums): x * 2 end
print(doubled)

# Test 2: for filter - keep only elements that satisfy predicate
evens = for filter(x from nums): num-modulo(x, 2) == 0 end
print(evens)

# Test 3: for fold - sum all elements
total = for fold(acc from 0, x from nums): acc + x end
print(total)

# Test 4: for each - side effects only
for each(x from nums):
  print(x)
end

# Test 5: for map with expression
squares = for map(n from [list: 1, 2, 3]): n * n end
print(squares)

check "for loop tests":
  # Map test
  for map(x from [list: 1, 2, 3]): x + 1 end is [list: 2, 3, 4]

  # Filter test
  for filter(x from [list: 1, 2, 3, 4]): x > 2 end is [list: 3, 4]

  # Fold test
  for fold(sum from 0, x from [list: 1, 2, 3]): sum + x end is 6

  # Map with more complex expression
  for map(x from [list: 10, 20, 30]): x / 10 end is [list: 1, 2, 3]
end
