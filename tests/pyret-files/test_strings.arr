# Test string operations

check "string concatenation":
  "hello" + " " + "world" is "hello world"
  "a" + "b" + "c" is "abc"
end

check "string equality":
  "hello" is "hello"
  "hello" is-not "world"
end

check "empty string":
  "" + "test" is "test"
  "test" + "" is "test"
end

check "string with numbers":
  "count: " + "5" is "count: 5"
end
