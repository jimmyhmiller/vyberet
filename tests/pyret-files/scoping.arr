# Test lexical scoping and shadowing

x = 10
print(x)

# Shadow with explicit keyword
fun test() block:
  shadow x = 20
  print(x)
  shadow x = 30
  print(x)
end

test()

# Original x is unchanged
print(x)

# Nested scopes
fun outer() block:
  shadow x = 100
  print(x)
  fun inner():
    shadow x = 200
    print(x)
  end
  inner()
  print(x)
end

outer()
print(x)
