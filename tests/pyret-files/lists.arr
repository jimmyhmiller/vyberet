# Test list construct expressions

# Empty list
empty-list = [list: ]
print(empty-list)

# Simple list
nums = [list: 1, 2, 3, 4, 5]
print(nums)

# List with expressions
x = 10
y = 20
calc = [list: x, y, x + y]
print(calc)

# Nested lists
nested = [list: [list: 1, 2], [list: 3, 4]]
print(nested)
