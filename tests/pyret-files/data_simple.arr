# Simple data declaration examples

data Color:
  | red
  | green
  | blue
end

data Point:
  | point(x, y)
end

data Maybe:
  | some(value)
  | none
end

# Test constructors
my-color = red()
my-point = point(3, 4)
my-maybe = some(42)
my-none = none()

print(my-color)
print(my-point)
print(my-maybe)
print(my-none)
