# Correct GCD using num-modulo
# This is the proper Pyret way, as shown in the Pyret docs

fun gcd(a, b):
  doc: "Calculates the greatest common divisor of a and b using Euclid's algorithm."
  if b == 0:
    a
  else:
    gcd(b, num-modulo(a, b))
  end
end

# Test cases
print(gcd(12, 8))   # Should print 4
print(gcd(48, 18))  # Should print 6
print(gcd(100, 25)) # Should print 25
print(gcd(17, 13))  # Should print 1 (coprime)
