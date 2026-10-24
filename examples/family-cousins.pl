% Identify cousin pairs in a supplied family tree by comparing generation
% numbers and inherited branch labels. In this model, people count as cousins
% when they share a generation but belong to different labeled branches. This
% is a classification for the given tree, rather than a general calculation
% of cousin degree or generations removed.

parent(adam, bob).
parent(adam, carol).
parent(bob, dave).
parent(bob, eve).
parent(carol, frank).
parent(carol, grace).
parent(dave, heidi).
parent(eve, ivan).
parent(frank, judy).
generation(adam, 0).
branch(dave, b).
branch(eve, b).
branch(frank, c).
branch(grace, c).

generation(Child, Next) :-
    parent(Parent, Child),
    generation(Parent, N),
    Next is N + 1.
branch(Child, Branch) :-
    parent(Parent, Child),
    branch(Parent, Branch).
cousin(X, Y) :-
    generation(X, N),
    generation(Y, N),
    branch(X, A),
    branch(Y, B),
    A \= B.

% Answer and Reason
?- cousin(X, Y).
?- why(cousin(heidi, judy)).
?- generation(heidi, N).
?- branch(judy, Branch).

% Check
:- assertion(cousin(dave, frank)).
:- assertion(cousin(frank, dave)).
:- assertion(cousin(heidi, judy)).
:- assertion(cousin(judy, heidi)).
:- assertion(\+ cousin(dave, eve)).
:- assertion(\+ cousin(dave, judy)).
:- assertion(\+ cousin(dave, dave)).
:- assertion(generation(heidi, 3)).
:- assertion(branch(judy, c)).
:- assertion(aggregate_all(count, cousin(X, Y), 12)).
