import lists as L

provide: test end

fun test():
  mylist = [list: 1, 2, 3, 4, 5]
  doubled = L.map(lam(x): x * 2 end, mylist)
  L.length(doubled)
end

print(test())
