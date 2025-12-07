import file("helper.arr") as H

provide: test end

fun test():
  H.double(5) + H.triple(3)
end

print(test())
