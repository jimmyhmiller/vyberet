;;; GCD Example Using Proper Integer Division
;;; This shows how GCD should work with R4RS-compliant Scheme

;; Load runtime library
(load "runtime/runtime.scm")

;; GCD using quotient and remainder (correct for all R4RS implementations)
(define (gcd-correct a b)
  (if (= b 0)
      a
      (gcd-correct b (pyret:num-remainder a b))))

;; Test cases
(pyret:print (gcd-correct 48 18))   ; Should be 6
(pyret:print (gcd-correct 12 8))    ; Should be 4
(pyret:print (gcd-correct 100 35))  ; Should be 5
(pyret:print (gcd-correct 17 19))   ; Should be 1 (coprime)
