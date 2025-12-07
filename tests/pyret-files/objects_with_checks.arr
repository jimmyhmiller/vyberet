# Object literals with check blocks

# Simple object
point = {x: 10, y: 20}

check:
  point.x is 10
  point.y is 20
end

# Nested object
person = {
  name: "Alice",
  age: 30,
  address: {
    street: "123 Main St",
    city: "Boston"
  }
}

check:
  person.name is "Alice"
  person.age is 30
  person.address.street is "123 Main St"
  person.address.city is "Boston"
end

# Empty object
empty = {}

# Object with different types
mixed = {
  num: 42,
  str: "hello",
  bool: true,
  nested: {inner: 99}
}

check:
  mixed.num is 42
  mixed.str is "hello"
  mixed.bool is true
  mixed.nested.inner is 99
end
