# Builtin Option Type
# This is a simplified builtin implementation
# Based on Pyret's option type but with corrected and-then behavior

provide: none, some, Option end

data Option:
  | none with:
    method or-else(self, v):
      v
    end,
    method and-then(self, f):
      self
    end
  | some(value) with:
    method or-else(self, v):
      self.value
    end,
    method and-then(self, f):
      # Call f and always wrap the result in some
      # This matches the trove implementation
      some(f(self.value))
    end
end

check "Option tests":
  none.or-else(1) is 1
  none.and-then(lam(x): some(x + 2) end) is none

  some(5).or-else(0) is 5
  some(5).and-then(lam(x): x + 2 end) is some(7)
  some(5).and-then(lam(x): some(x + 2) end) is some(7)
end
