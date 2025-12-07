# Test Method Call Syntax
# Tests the new obj.method(args) syntax for built-in types

check "List methods":
  # Test length()
  nums = [list: 1, 2, 3, 4, 5]
  nums.length() is 5

  # Test empty list length
  empty = [list: ]
  empty.length() is 0

  # Test first()
  nums.first() is 1

  # Test rest()
  rest = nums.rest()
  rest.length() is 4
  rest.first() is 2

  # Test get()
  nums.get(0) is 1
  nums.get(2) is 3
  nums.get(4) is 5

  # Test reverse()
  rev = nums.reverse()
  rev.first() is 5
  rev.get(4) is 1

  # Test append()
  more = [list: 6, 7]
  combined = nums.append(more)
  combined.length() is 7
  combined.get(5) is 6
  combined.get(6) is 7
end

check "String methods":
  # Test length()
  text = "hello"
  text.length() is 5

  # Test empty string
  empty = ""
  empty.length() is 0

  # Test substring()
  text.substring(0, 2) is "he"
  text.substring(1, 4) is "ell"
  text.substring(0, 5) is "hello"

  # Test char-at()
  text.char-at(0) is "h"
  text.char-at(4) is "o"

  # Test split()
  csv = "a,b,c"
  parts = csv.split(",")
  parts.length() is 3
  parts.get(0) is "a"
  parts.get(1) is "b"
  parts.get(2) is "c"

  # Test contains()
  text.contains("ell") is true
  text.contains("xyz") is false

  # Test to-upper()
  text.to-upper() is "HELLO"

  # Test to-lower()
  upper = "WORLD"
  upper.to-lower() is "world"

  # Test repeat()
  "ha".repeat(3) is "hahaha"
  "x".repeat(0) is ""
  "ab".repeat(2) is "abab"

  # Test trim()
  "  hello  ".trim() is "hello"
  "\thello\n".trim() is "hello"
  "hello".trim() is "hello"
end

check "Chained method calls":
  # Chain list methods
  nums = [list: 1, 2, 3, 4, 5]
  nums.rest().rest().first() is 3
  nums.reverse().first() is 5

  # Chain string methods
  text = "  HELLO  "
  text.trim().to-lower() is "hello"

  # Split and get length
  "a,b,c,d".split(",").length() is 4
end

check "Method calls with expressions":
  # Method on expression result
  ([list: 1, 2, 3]).length() is 3
  ("hello" + "world").length() is 10

  # Method with expression arguments
  nums = [list: 10, 20, 30]
  nums.get(1 + 1) is 30

  "hello".substring(0, 2 + 1) is "hel"
end
