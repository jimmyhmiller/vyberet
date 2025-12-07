# Test tuple literals and access

# Create a tuple
t = {1; 2; 3}

# Access elements
print(t.{0})
print(t.{1})
print(t.{2})

# Nested tuples
nested = {{10; 20}; {30; 40}}
print(nested.{0}.{0})
print(nested.{0}.{1})
print(nested.{1}.{0})
print(nested.{1}.{1})

# Tuples with expressions
x = 5
y = 10
point = {x; y; x + y}
print(point.{0})
print(point.{1})
print(point.{2})
