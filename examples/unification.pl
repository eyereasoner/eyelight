% Use matching term structure and repeated variables to constrain answers.
% The append rules split the complete list [a, b] into every prefix and
% suffix, including empty lists. The pair(X, X) pattern requires both entries
% to match, and [Head | Tail] separates a list's first item from the
% remaining items.

:- table append/3.

append([], Ys, Ys).
append([X | Xs], Ys, [X | Zs]) :-
    append(Xs, Ys, Zs).

matching_pair(pair(X, X)).
head_tail([Head | Tail], Head, Tail).

% Answer and Reason
?- append(Prefix, Suffix, [a, b]).
?- why.
?- matching_pair(pair(same, same)).
?- why.
?- matching_pair(pair(a, b)).
?- head_tail([a, b, c], Head, Tail).
?- why.

% Check
:- assertion(append([], [a, b], [a, b])).
:- assertion(append([a], [b], [a, b])).
:- assertion(append([a, b], [], [a, b])).
:- assertion(aggregate_all(count, append(Prefix, Suffix, [a, b]), 3)).
:- assertion(matching_pair(pair(same, same))).
:- assertion(\+ matching_pair(pair(a, b))).
:- assertion(head_tail([a, b, c], a, [b, c])).
