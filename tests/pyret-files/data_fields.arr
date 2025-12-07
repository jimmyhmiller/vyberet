# Test field access for data variants

data Point:
  | point(x, y)
end

data Person:
  | person(name, age)
end

p = point(3, 4)
print(p.x)
print(p.y)

alice = person("Alice", 30)
print(alice.name)
print(alice.age)
