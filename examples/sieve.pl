% Find primes up to 100 with the sieve of Eratosthenes. Start with the
% integers from 2 through the limit, keep the first remaining number, and
% remove its multiples before continuing.
%
% The supplied limit keeps the explanation manageable. Each filtering step
% records its intermediate list so the result can be followed and checked.

:- table primes/2, range/3, sift/2, remove/3.

primes(Limit, Primes) :-
    range(2, Limit, Integers),
    sift(Integers, Primes).

range(Start, End, []) :-
    Start > End.
range(Start, End, [Start | Rest]) :-
    Start =< End,
    Next is Start + 1,
    range(Next, End, Rest).

sift([], []).
sift([I | Is], [I | Ps]) :-
    remove(I, Is, New),
    sift(New, Ps).

remove(_, [], []).
remove(P, [I | Is], Rest) :-
    0 =:= I mod P,
    remove(P, Is, Rest).
remove(P, [I | Is], [I | Rest]) :-
    0 =\= I mod P,
    remove(P, Is, Rest).

% Answer and Reason
?- primes(100, Primes).
?- why.

% Check
:- assertion(primes(1, [])).
:- assertion(primes(2, [2])).
:- assertion(primes(10, [2, 3, 5, 7])).
:- assertion(primes(100,
    [2, 3, 5, 7, 11, 13, 17, 19, 23, 29, 31, 37, 41, 43,
     47, 53, 59, 61, 67, 71, 73, 79, 83, 89, 97])).
