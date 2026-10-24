% Generate Fibonacci numbers: start with 0 and 1, then add the previous two
% numbers to obtain the next. Supply a nonnegative integer index.
%
% Tabling reuses previously calculated answers. The queries include F(10000),
% whose integer value is kept exactly, and the checks verify a relation
% between three neighboring Fibonacci numbers known as Cassini's identity.

:- table fibonacci/2.
fibonacci(0, 0).
fibonacci(1, 1).

fibonacci(Index, _Calc3) :-
    Index > 1,
    _Calc1 is (Index - 1),
    fibonacci(_Calc1, Previous),
    _Calc2 is (Index - 2),
    fibonacci(_Calc2, Earlier),
    _Calc3 is (Previous + Earlier).
cassini(10000, _Calc1) :-
    fibonacci(10000, Current),
    fibonacci(9998, Earlier),
    fibonacci(9999, Previous),
    _Calc1 is ((Current * Earlier) - (Previous * Previous)).

% Answer and Reason
?- fibonacci(0, What).
?- fibonacci(1, What).
?- fibonacci(2, What).
?- fibonacci(3, What).
?- fibonacci(4, What).
?- fibonacci(5, What).
?- fibonacci(6, What).
?- fibonacci(7, What).
?- fibonacci(8, What).
?- fibonacci(9, What).
?- fibonacci(10, What).
?- why((fibonacci(5, 5))).
?- fibonacci(10000, What).

% Check
:- assertion((fibonacci(0, 0))).
:- assertion((fibonacci(1, 1))).
:- assertion((fibonacci(10, 55))).
:- assertion((fibonacci(100, 354224848179261915075))).
:- assertion((cassini(10000, -1))).
