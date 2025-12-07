# Test list construction

check "empty list":
  lst = [list: ]
  lst is [list: ]
end

check "list with single element":
  lst = [list: 42]
  lst is [list: 42]
end

check "list with multiple elements":
  lst = [list: 1, 2, 3, 4, 5]
  lst is [list: 1, 2, 3, 4, 5]
end

check "list with strings":
  lst = [list: "a", "b", "c"]
  lst is [list: "a", "b", "c"]
end

check "list with mixed expressions":
  a = 10
  b = 20
  lst = [list: a, a + b, b * 2]
  lst is [list: 10, 30, 40]
end

check "nested lists":
  lst = [list: [list: 1, 2], [list: 3, 4]]
  lst is [list: [list: 1, 2], [list: 3, 4]]
end

check "list-set construction":
  s = [list-set: 1, 2, 3]
  s is [list-set: 1, 2, 3]
end
