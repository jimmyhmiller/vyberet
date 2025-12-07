# Comprehensive data declaration tests

# Simple singleton variants
data Color:
  | red
  | green
  | blue
end

# Data with fields
data Point:
  | point(x, y)
end

# Data with multiple variants
data Shape:
  | circle(radius)
  | rectangle(width, height)
  | triangle(base, height)
end

# Data with optional-like pattern
data Maybe:
  | some(value)
  | none
end

# Data with list-like pattern
data List:
  | empty
  | link(first, rest)
end

# Test singleton constructors
c1 = red()
c2 = green()
c3 = blue()

print(c1)
print(c2)
print(c3)

# Test constructors with fields
p1 = point(3, 4)
p2 = point(0, 0)

print(p1)
print(p2)

# Test field access
print(p1.x)
print(p1.y)
print(p2.x)
print(p2.y)

# Test multiple variants
s1 = circle(5)
s2 = rectangle(10, 20)
s3 = triangle(6, 8)

print(s1)
print(s2)
print(s3)

print(s1.radius)
print(s2.width)
print(s2.height)
print(s3.base)
print(s3.height)

# Test maybe
m1 = some(42)
m2 = none()

print(m1)
print(m2)
print(m1.value)

# Test list
l1 = empty()
l2 = link(1, empty())
l3 = link(2, link(1, empty()))

print(l1)
print(l2)
print(l3)
print(l2.first)
print(l3.first)
