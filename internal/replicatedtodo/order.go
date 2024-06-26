package replicatedtodo

import (
	"bytes"
	"fmt"
)

type errorString string

func (e errorString) Error() string { return string(e) }

const (
	alphabetLow  = 'a'
	alphabetHigh = 'z'

	alphabetPred = alphabetLow - 1
	alphabetSucc = alphabetHigh + 1

	ErrInvalidArgument = errorString("invalid argument")
)

type OrderString []byte

func validByte(b byte) bool {
	return b >= alphabetLow && b <= alphabetHigh
}

func validBytes(bytes []byte) bool {
	// Reject strings with bytes lying outside the allowed alphabet.
	for _, b := range bytes {
		if b < alphabetLow || b > alphabetHigh {
			return false
		}
	}

	// Reject strings that end with the alphabet's lowest character,
	// since these could not have been generated by this code
	// and may have no solution.
	if len(bytes) > 0 && bytes[len(bytes)-1] == alphabetLow {
		return false
	}
	return true
}

func (s OrderString) Valid() bool {
	return validBytes(s)
}

func MidString(lower, upper OrderString) (OrderString, error) {
	// Algorithm taken from https://stackoverflow.com/a/38927158
	if !validBytes(lower) || !validBytes(upper) {
		return nil, fmt.Errorf(
			"invalid byte value(s): %w", ErrInvalidArgument)
	}

	// Verify that left < right (lexicographically)
	if len(upper) > 0 && bytes.Compare(lower, upper) >= 0 {
		return nil, fmt.Errorf(
			"left is not less than right: %w", ErrInvalidArgument)
	}

	p := byte(0)
	n := byte(0)

	var res []byte

	for p == n {
		p = alphabetPred
		if len(lower) > 0 && len(res) < len(lower) {
			p = lower[len(res)]
		}

		n = alphabetSucc
		if len(upper) > 0 && len(res) < len(upper) {
			n = upper[len(res)]
		}

		if p == n {
			res = append(res, p)
		}
	}

	if p == alphabetPred {
		// Left is a prefix of Right
		//
		// While Right's next character is the first character of the alpabet....
		for n == alphabetLow {
			// Append the first character to match.
			res = append(res, alphabetLow)
			// Get Right's next charater.
			if len(res) < len(upper) {
				n = upper[len(res)]
			} else {
				n = alphabetSucc
			}
		}
		// If Right's next character is the second character of the alphabet...
		if n == alphabetLow+1 {
			// Append the first character of the alphabet and set r to
			// one past the last character of the alphabet.
			res = append(res, alphabetLow)
			n = alphabetSucc
		}
	} else if p+1 == n {
		// Found consecutive byte values.
		res = append(res, p)
		n = alphabetSucc
		for {
			if len(res) >= len(lower) {
				p = alphabetPred
			} else {
				p = lower[len(res)]
			}
			if p != alphabetHigh {
				break
			}
			res = append(res, alphabetHigh)
		}
	}

	res = append(res, n-(n-p)/2)

	return res, nil
}
