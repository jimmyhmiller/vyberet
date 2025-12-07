import file("combined-utils.arr") as Utils
import lists as L

provide: * end

fun test():
  banner = Utils.make-banner(5)
  nums = [list: 1, 2, 3]
  total = L.fold(lam(x, acc): x + acc end, 0, nums)
  total
end

print(test())
