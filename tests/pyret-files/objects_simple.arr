# Simple object literal test

# Create an object
point = {x: 10, y: 20}

# Access fields
print(point.x)
print(point.y)

# Nested objects
person = {
  name: "Alice",
  age: 30,
  address: {
    street: "123 Main St",
    city: "Boston"
  }
}

print(person.name)
print(person.age)
print(person.address.street)
print(person.address.city)
