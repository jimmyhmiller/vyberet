# Test Pyret numeric functions
# NOTE: These functions require importing the numbers library in real Pyret
# This is a test for our compiler's runtime support

# Division returns exact rationals in Pyret
print(48 / 18)  # 8/3

# Modulo is available in standard Pyret
print(num-modulo(48, 18))     # 12 (modulo)
print(num-modulo(-5, 3))      # 1 (in Scheme's modulo)
