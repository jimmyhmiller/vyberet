# Pyret Compiler

A compiler for [Pyret](https://www.pyret.org/) targeting Scheme, vibe coded with Claude.

## Note

This was pretty much entirely vibe coded with Claude. All example files used in the tests came from [pyret-lang](https://github.com/brownplt/pyret-lang/) and are under the Apache License.

When I handed it to [@tekknolagi](https://github.com/tekknolagi) he immediately found that even an example from the Pyret home page didn't parse. So caveat emptor.

## Usage

```bash
cargo run --bin compile tests/pyret-files/factorial.arr --run
```

## Compiled Language Features

### Functions and Lambdas
```pyret
fun factorial(n):
  if n <= 1: 1
  else: n * factorial(n - 1)
  end
end

double = lam(x): x * 2 end
```

### Data Declarations and Pattern Matching
```pyret
data Shape:
  | circle(radius)
  | rectangle(width, height)
end

fun area(s):
  cases (Shape) s:
    | circle(r) => 3 * r * r
    | rectangle(w, h) => w * h
  end
end
```

### Lists and For Loops
```pyret
nums = [list: 1, 2, 3, 4, 5]

doubled = for map(x from nums): x * 2 end
evens = for filter(x from nums): num-modulo(x, 2) == 0 end
total = for fold(acc from 0, x from nums): acc + x end
```

### Method Calls
```pyret
nums = [list: 1, 2, 3]
nums.length()   # 3
nums.first()    # 1
nums.rest()     # [list: 2, 3]
nums.get(0)     # 1
nums.reverse()  # [list: 3, 2, 1]

str = "hello world"
str.length()           # 11
str.substring(0, 5)    # "hello"
str.split(" ")         # [list: "hello", "world"]
```

### Objects
```pyret
point = {x: 10, y: 20}
point.x  # 10

person = {
  name: "Alice",
  address: { city: "Boston" }
}
person.address.city  # "Boston"
```

### Check Blocks (Testing)
```pyret
check "arithmetic":
  (2 + 2) is 4
  (3 * 4) is 12
end

check "lists":
  [list: 1, 2].length() is 2
end
```

### Other Features
- Tuples: `{1; "hello"; true}`
- Mutable variables: `var x = 0` then `x := x + 1`
- Blocks: `block: ... end`
- When expressions: `when x > 0: print(x) end`
- All binary operators: `+`, `-`, `*`, `/`, `<`, `<=`, `>`, `>=`, `==`, `<>`, `and`, `or`

## Scheme Backends

Compiles to R4RS Scheme. Tested with:
- Chicken Scheme (default)
- Gambit Scheme: `--interpreter gsi`
- Chez Scheme: `--interpreter chez`
- Ribbit Scheme: `--interpreter ribbit`

## Not Yet Implemented

- Object methods with `self`
- Object extension `obj.{z: 3}`
- Tables, reactors, refinements
- Type checking (annotations are parsed but ignored)
