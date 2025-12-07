# Test mutable variables (var and :=)

# Create mutable variable
var x = 5
print(x)

# Mutate it
x := 10
print(x)

# Use in expression
x := x + 5
print(x)

# Multiple vars
var a = 1
var b = 2
print(a)
print(b)

a := a + b
print(a)

b := a * 2
print(b)
