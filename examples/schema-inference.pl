% Infer additional classifications and relationships from a small schema.
% Cats are mammals and mammals are animals, so Koko's cat classification also
% implies both broader classifications.
%
% Likewise, a parent_of link implies related_to and connected_to links. The
% domain and range rules classify the source and target of a parent_of link
% as people. These are the schema rules explicitly implemented below.

subclass(cat, mammal).
subclass(mammal, animal).
subproperty(parent_of, related_to).
subproperty(related_to, connected_to).
domain(parent_of, person).
range(parent_of, person).
type(koko, cat).
triple(alice, parent_of, bob).

subclass(A, C) :-
    subclass(A, B),
    subclass(B, C).
subproperty(P, R) :-
    subproperty(P, Q),
    subproperty(Q, R).
type(X, B) :-
    type(X, A),
    subclass(A, B).
triple(S, Q, O) :-
    triple(S, P, O),
    subproperty(P, Q).
type(S, Class) :-
    triple(S, P, _),
    domain(P, Class).
type(O, Class) :-
    triple(_, P, O),
    range(P, Class).

% Answer and Reason
?- type(Who, Class).
?- why(type(koko, animal)).
?- triple(Subject, Property, Object).
?- why(triple(alice, connected_to, bob)).
?- type(alice, person).
?- why.

% Check
:- assertion(subclass(cat, animal)).
:- assertion(subproperty(parent_of, connected_to)).
:- assertion(type(koko, mammal)).
:- assertion(type(koko, animal)).
:- assertion(type(alice, person)).
:- assertion(type(bob, person)).
:- assertion(triple(alice, related_to, bob)).
:- assertion(triple(alice, connected_to, bob)).
:- assertion(aggregate_all(count, triple(Subject, Property, Object), 3)).
:- assertion(aggregate_all(count, type(Who, Class), 5)).
