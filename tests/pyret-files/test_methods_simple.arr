# Simple test of method call syntax without check blocks

# Test list methods
nums = [list: 1, 2, 3, 4, 5]
print("List length:")
print(nums.length())

print("First element:")
print(nums.first())

print("Rest of list:")
rest = nums.rest()
print(rest)

print("Get index 2:")
print(nums.get(2))

# Test string methods
text = "hello"
print("String length:")
print(text.length())

print("Substring 0-2:")
print(text.substring(0, 2))

print("Split test:")
csv = "a,b,c"
parts = csv.split(",")
print(parts)
print("Parts length:")
print(parts.length())

print("To upper:")
print(text.to-upper())

print("Repeat:")
print("ha".repeat(3))

print("Done!")
